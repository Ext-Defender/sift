# SIFT

## About
Sift is a multi-threaded regex pattern finding tool capable of scanning most popular file types. It can scan for multiple patterns, and patterns are encrypted and saved to a config file for convenience.

If there are any findings during a scan they will be output to the user specified directory as in a <b>CSV</b> file. Upon first scan a log file will also be generated in this directory that will be populated with any errors generated during the scan.

For a scan to run:
- At least one pattern needs to be defined
- At least one root directory needs to be defined
- Output directory must be defined

## Getting Started
- Run the sift executable, and enter your desired password.
- Add your space delimited patterns using the <i>-a</i> flag.
- Add your desired output directory using the <i>-o</i> flag.
- Add your space delimited root directories using the <i>-r</i> flag.
- Run your first scan using the <i>-s</i> flag


This will run a scan of my <i>D</i> drive looking for the pattern <i>test</i> and output the findings to a csv file inside the folder <i>C:\test</i>
```console
.\sift.exe -a test -o C:\test -r D:\ -s
```
## Flags

|flag| Description|
|---|---|
|-a| Add a pattern to the config file|
|-A| Remove a pattern from the config file|
|-k| Print patterns to console|
|-r| Add a root to the config file|
|-R| Remove a root to the config file|
|-m| Print roots to console|
|-o| Modify the output location|
|-l| Print the output directory to console|
|-z| Print the config file to console|
|-q| Reset the config file|
|-i| Make scan case sensitive (Scans are case-insensitive by default)|
|-v| Verbose output|

## Troubleshooting
### Forgot Password
Use the <i>-q</i> flag to reset the config file, and enter a new password. This will also lose all the previously entered patterns, roots, and output directory.
