# heaven-on-earth
`heaven-on-earth` is a small command-line utility for querying images from [/r/EarthPorn](https://www.reddit.com/r/EarthPorn/) and setting them as a wallpaper.

### Example Usage
`heaven-on-earth run --mode=top --span=day --run-every="0 * * * *"`  
`heaven-on-earth run --mode=new --min-ratio="12/9" --max-ratio="20/9"`  
`heaven-on-earth install --mode=top --span=day`   
*see `heaven-on-earth --help` for a full list of arguments*

### Additional info
See [this](https://crontab.guru) for `--run-every` syntax  
Tested on Windows 10, should work on unix with `feh` installed  