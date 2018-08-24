# heaven-on-earth
[![Build Status](https://travis-ci.org/NyxCode/heaven-on-earth.svg?branch=master)](https://travis-ci.org/NyxCode/heaven-on-earth)  

`heaven-on-earth` is a small command-line utility for querying images from [/r/EarthPorn](https://www.reddit.com/r/EarthPorn/) and setting them as a wallpaper.

### Example Usage
**`heaven-on-earth run --mode=top --span=day`**  
*=> downloads and sets the top post of the last 24 hours as your background*
  
**`heaven-on-earth run --mode=new --min-ratio="12/9" --max-ratio="20/9"`**  
*=> gets the newest image with a ratio between 12/9 and 20/9*

**`heaven-on-earth run --mode=top --span=day --query-size=50 --random`**  
*=> gets one of the 50 top images of the day*

**`heaven-on-earth run --mode=controversial --span=hour --run-every="0 * * * *"`**  
*=> gets the most controversial image of the past hour, every hour*

**`heaven-on-earth install --mode=top --span=day`**   
*=> runs `heaven-on-earth run --mode=top --span=day` every time you log in (log out/in required)*

### Platform support
Windows  
Unix *(with `feh` installed)*  
MacOS

### Additional info
See [this](https://crontab.guru) for `--run-every` syntax  
My setup: `heaven-on-earth install --mode=top --span=hour --run-every"0 * * * *" --query-size=50 ----min-ratio="12/9" --max-ratio="20/9" --random`
