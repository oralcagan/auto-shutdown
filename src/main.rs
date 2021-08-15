use std::fs;
use std::io;
use std::io::Write;
use std::process;
use std::time;

macro_rules! res_match {
    ($a:expr, $b:expr) => {
        match $a {
            Ok(t) => t,
            Err(_) => $b,
        }
    };
}

macro_rules! op_match {
    ($a:expr, $b:expr) => {
        match $a {
            Some(t) => t,
            None => $b,
        }
    };
}

/*
To get the name of the game, the program requests the html page https://store.steampowered.com/app/<app_id> .
The name of the game is in this html tag: <div id="appHubAppName" class="apphub_AppName">{GAME_NAME}</div>,
So, the program reads from the http response stream until it matches 'class="apphub_AppName">', then it reads the stream and stores the data until it matches '</div>'.

To check if the game -or an update- is downloaded or not, this looks for the folders inside DOWNLOAD_FOLDER.
If the folder whose name is the id of the game isn't there, it concludes that the game has been downloaded.

You might have to change these constants for your own computer.
*/

///Explained above
const APP_NAME_PREFIX: &[u8] = b"<div id=\"appHubAppName\" class=\"apphub_AppName\">";
const APP_NAME_SUFFIX: &[u8] = b"</div>";
/// App page url
const APP_URL: &str = "https://store.steampowered.com/app/";
/// When we request an app page on steam, it might forward us to an age check page. To prevent this, we pass the request with these cookies.
const COOKIE_VAL: &str = "wants_mature_content=1; lastagecheckage=1-0-1999; birthtime=912463201";
/// Steam apps download folder.
const DOWNLOAD_FOLDER: &str = "C:\\Program Files (x86)\\Steam\\steamapps\\downloading";
/// Determines how frequently the program should check the "downloading" folder. The interval time is in seconds.
const CHECK_INTERVAL_SEC: u64 = 30;
/// Shutdown command name
const SHUTDOWN_CMD: &str = "shutdown";
/// Shutdown command args
const SHUTDOWN_ARGS: &[&str] = &["/s"];

fn main() {
    let id = prompt_user();
    if !look_for_folder(&id) {
        println!("Game download \"{}\" doesn't exist", id);
        return;
    }
    println!("Auto shutdown for {}", id);
    loop {
        std::thread::sleep(time::Duration::from_secs(CHECK_INTERVAL_SEC));
        if !look_for_folder(&id) {
            shutdown();
            break;
        }
    }
}

fn shutdown() {
    process::Command::new(SHUTDOWN_CMD)
        .args(SHUTDOWN_ARGS)
        .output()
        .unwrap();
}

fn prompt_user() -> String {
    let client = ureq::agent();
    let mut msg = String::from("");
    for f in get_folders() {
        let name = get_game_name(&f, &client);
        let temp = format!("{} - {}\n", &f, &name);
        msg.push_str(&temp);
    }
    println!("{}", msg);
    get_app_id_input()
}

fn get_game_name(id: &str, client: &ureq::Agent) -> String {
    let mut reader = res_match!(
        req_game_page(id, &client),
        return "Unknown game".to_string()
    );
    if reader_match_exp(APP_NAME_PREFIX, &mut reader) {
        return res_match!(
            String::from_utf8(op_match!(
                reader_read_and_store_until(APP_NAME_SUFFIX, &mut reader),
                return "Unknown game".to_string()
            )),
            return "Unknown game".to_string()
        );
    }
    "Unknown Game".to_string()
}

fn make_app_page(id: &str) -> String {
    let mut url = APP_URL.to_owned();
    url.push_str(id);
    url
}

fn req_game_page(id: &str, client: &ureq::Agent) -> Result<impl io::Read + Send, ureq::Error> {
    let url = make_app_page(id);
    let res = client
        .get(&url)
        .set("Cookie", COOKIE_VAL)
        .set("Connection", "Keep-Alive")
        .set("Keep-Alive", "timeout=30");
    match res.call() {
        Ok(r) => return Ok(r.into_reader()),
        Err(e) => return Err(e),
    }
}

fn reader_match_exp(exp: &[u8], reader: &mut impl io::Read) -> bool {
    let l = exp.len();
    let mut buf = vec![0; l];
    res_match!(reader.read_exact(&mut buf), return false);
    if &buf == exp {
        return true;
    }
    loop {
        buf.rotate_left(1);
        res_match!(reader.read_exact(&mut buf[l - 1..l]), return false);
        if buf == exp {
            return true;
        }
    }
}

fn reader_read_and_store_until(exp: &[u8], reader: &mut impl io::Read) -> Option<Vec<u8>> {
    let mut v = Vec::new();
    let l = exp.len();
    let mut buf = vec![0; l];
    res_match!(reader.read_exact(&mut buf), return None);
    if &buf == exp {
        return None;
    }
    v.extend_from_slice(&buf);
    loop {
        buf.rotate_left(1);
        res_match!(reader.read_exact(&mut buf[l - 1..l]), return None);
        if &buf == exp {
            rem_n_from_end(&mut v, l - 1);
            return Some(v);
        }
        v.extend_from_slice(&mut buf[l - 1..l]);
    }
}

fn get_folders() -> Vec<String> {
    let folder_iter = res_match!(fs::read_dir(DOWNLOAD_FOLDER), return Vec::new());
    folder_iter
        .filter(|x| {
            let entry = res_match!(x, {
                println!("Couldn't read a folder");
                return false;
            });
            res_match!(entry.metadata(), {
                println!("Couldn't read a folder");
                return false;
            })
            .is_dir()
        })
        .map(|x| {
            let entry = res_match!(x, { return "Unknown folder".to_string() });
            op_match!(entry.file_name().to_str(), { "Unknown folder" }).to_string()
        })
        .collect()
}

fn look_for_folder(folder: &str) -> bool {
    let folder_iter = res_match!(fs::read_dir(DOWNLOAD_FOLDER), return false);
    for f in folder_iter {
        let entry = res_match!(f, {
            println!("Couldn't read a folder");
            continue;
        });
        let osstr = entry.file_name();
        let a_folder = op_match!(osstr.to_str(), continue);
        if a_folder == folder {
            return true;
        }
    }
    return false;
}

fn get_app_id_input() -> String {
    use std::io::BufRead;
    print!("Select a game: ");
    let _ = std::io::stdout().flush();
    io::stdin()
        .lock()
        .lines()
        .next()
        .expect("Couldn't read the input")
        .expect("Couldn't read the input")
}

fn rem_n_from_end<T>(v: &mut Vec<T>, n: usize) {
    for _ in 0..n {
        v.pop();
    }
}
