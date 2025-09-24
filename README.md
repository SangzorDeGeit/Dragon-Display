# Dragon-Display
Application based on Rust to display images on a second screen, suited for Dungeons &amp; Dragons

# Using Google Drive
In order to use google drive you need to get a client secret from Google and make sure that some values are set correctly in this file.
## Configuring the Client secret
To get a client secret follow the following steps:
1. Go to [https://console.cloud.google.com] and log in with your google account  
2. Create a project if prompted  
3. Click on the navigation menu (top left) -> APIs & Services -> Library  
5. In the searchbar search for 'Google Drive API' and click on Google Drive API  
6. Click on enable  
7. Follow the steps to configure an oauth consent screen  
8. After oauth consent screen is configured you may need navigate to APIs & Services -> oauth consent screen  
9. Click on Audience (left bar) and add a user (give your gmail)  
10. Navigate to APIs & Services -> credentials  
11. Click Create credentials (top) -> Oauth Client Id  
12. Select Application type: Desktop app.  
13. Give it a name (like PC-<yourname>)  
14. In the pop up window click 'Download JSON'
15. Rename the downloaded file to 'client_secret.json'
16. Edit the file: change the value of redirect_uris to: ["`https://localhost:8000`"] (add ':8000' after localhost)  
17. Put the file in the directory from where you run the program  

## Setup folder in your drive
Create a folder in your google drive in which you put all the images that you want to display using Dragon-Display. When adding a campaign you can select this folder. Dragon display will use the selected folder to synchronize images to a local folder


