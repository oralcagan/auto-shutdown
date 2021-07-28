Automatically shuts down the computer when a steam download ends.  
You might need to change the constants in src/main.rs for this to work on your computer.  

There is an http client library in the dependencies because the program fetches the name of the games from their app id.  

### How does it work?
Basically, a folder is created inside the folder Steam/steampps/downloading, with a name as your game's app id,  
when you start downloading a game or an update. This folder is used as a temporary storage by Steam until all of your game files  
are downloaded. When the work is done, steam deletes this folder. This is how autoshutdown understands whether the game is downloaded or not.  
It just checks the folder in a set interval to look if that folder still exists. If it doesn't exist program runs the shutdown command.  