# dust-manipulator
An assortment of programs intended for use in UNDERTALE speedruns.

Currently unfinished and experimental code with some of its programs, but is currently usable for naming screen RNG seed finding and Dogi skip (marriage manipulator).

## Install and setup instructions
- Install the OBS plugin
    * You may need to update to a more recent version of OBS. As of writing, the minimum required is at least OBS 31.1.0.
    * Download the ZIP from <https://github.com/colinator27/dust-manipulator-plugin/releases>, use your program of choice to copy the DLL (or .so on Linux) file contained within to your OBS plugins directory.
        - (On Windows, the plugin directory can be `C:\Program Files\obs-studio\obs-plugins\64bit`, but it may be different depending on how OBS was installed.)
- Install the actual program
    * Download and extract the ZIP from <https://github.com/colinator27/dust-manipulator/releases>, to any location you want.
    * Optional: you can use a text editor to edit `config.json`. This contains:
        - `runner_version`: The effective runner version of Undertale that you want to run with. Valid values are:
            * `Undertale_Windows_v1_0`: The original release of the game, Windows-only.
            * `Undertale_Windows_v1_001`: The Windows runner shipped with 1.001 Windows, or any of the modified speedrun versions.
                - (This is what you should use if you run 1.001 Linux on Windows, and this is the default for Windows in general.)
            * `Undertale_Linux_v1_001`: The Linux runner shipped with 1.001 Linux.
                - (This is the default for Linux in general.)
            * `Undertale_Windows_v1_08`: Any runner versions beyond 1.001, currently.
        - `server_port`: Port that the tool will run a local server on. This port should be kept private in firewall settings if necessary.
            * If this is changed, it will need to be updated in the OBS plugin's filter settings as well.
            * Only one server/tool can run on a port at a given time.
            * Default port is set to 48654.
        - `hotkey_*_name`: Hotkey names to display in the tool.
            * These do *not* change the actual hotkeys, as those are configured in OBS global hotkey settings.
            * These are purely for visual display inside of the tool, and should be updated whenever the corresponding OBS hotkeys get changed.
- Setup for OBS
    * It's recommended to create a "group" in an OBS scene, placing any Undertale captures within.
        - Make sure that the group is exactly 4:3 aspect ratio (like Undertale itself), otherwise things will not work very well.
        - The group/capture sources must be enabled for the plugin to be enabled.
        - Display captures (and other types of captures) are totally fine to use, as long as they are still cropped to *only* the game, with no offsets/padding on the sides.
    * Add the "Dust Manipulator" effect filter to the group. It has these settings:
        - `Number of screenshots to take` (default: 10) - Used for rapid screenshots when doing dust particle RNG manipulation. Otherwise has no effect.
        - `Screenshot width` / `Screenshot height` (default: 640x480) - Used for sizing the screenshots sent to the tool. Should be left at default values for things to work, currently.
        - `Local port number to send data to` (default: 48654) - Should match the server port number in `config.json` from the tool.
	* It's *not* recommended to use more than one filter at a time, as this may lead to strange behavior.
    * Set your OBS hotkeys in the OBS settings. The defaults are F1, F2, F3, and F5, as reflected in `config.json` from the tool.
- Troubleshooting
	* You can view OBS logs from within the OBS interface; the plugin outputs some network-related stuff and errors there.
	* Generally, viewing the console logs from the tool itself can show what's happening internally.

## General usage
- The tool is split into multiple programs, some of which share data with each other (such as RNG seed/position).
- Press ESC while the tool window is focused to exit any program, or the entire tool itself.
- You can see the number of RNG matches in the tool's console output for troubleshooting purposes.

## Performing Dogi skip
- Open the "Naming Seed Search" program. You should see "Connected to OBS" when the plugin's filter is active (pressing any plugin hotkey will re-attempt a connection).
- Initial setup (RNG seed search):
    * Launch Undertale. Progress the intro story panels quickly; you can pause on the title card/instructions if needed.
    * Navigate the naming screen itself and choose a name at a brisk pace, preferably within 5 seconds to be safe.
        - While the shaking letters are visible, press hotkey 1 to take a screenshot from the OBS plugin.
    * The tool window now contains pixels. When you have downtime (e.g. the long hallway in Ruins), click/drag on all of the white pixels.
        - You can press hotkey 2 to teleport your cursor to the middle of the tool window, which also keeps focus on Undertale.
            * (This may not function 100% correctly if the window title is not exactly `UNDERTALE`, at least on Windows.)
        - Hotkey 4 can be used to re-focus the tool window, if desired (e.g. to press ESC to quit the program).
    * When done filling the pixels precisely, press hotkey 3 to perform the RNG search. This should be pretty quick.
    * Press hotkey 3 again to progress to the Marriage Manipulator tool.
- Marriage Manipulation:
    * Upon entering this tool, it will take up to 1-2 minutes to calculate/preload snowball data. The text in the bottom left will disappear upon completion.
        - (This is done slowly in a background thread, so as to not cause any lagspikes. It can be safely canceled with ESC like normal.)
    * During the run, you must not call excessive amounts of RNG. It's somewhat lenient, but on-screen textboxes call a *lot* of RNG every frame.
    * Perform Dogi skip using the standard speedrun setup, except no save/load is required (re-entering the room *is* still required).
        - Hotkey 1 can be used to take screenshots *and* teleport the mouse to the tool window (along with focusing Undertale, again).
    * You must click on the snowballs with slightly more precision than the old tool; it's zoomed in for convenience.
        - You should be accurate within about 1-2 pixels; if any snowball is placed too far away, matches may fail.
        - Right click clears any already-placed snowballs.
        - Using debug mode, you can repeatedly attempt the strat by re-entering the room (to an extent, before preloaded data runs out).

## Contributing
As this tool is currently unfinished, the code quality and structure is a bit all over the place. PRs to improve this are very welcome, so long as they don't conflict with anything being worked on.

## Build instructions
For the main program itself, the regular Rust build pipeline should work fine. There is a large dependency on SDL3 and related libraries.

The GitHub Actions workflows show all of the necessary steps for building and packaging full versions of the tool, for supported platforms.

To compile the shaders used in multiple programs, see the [shaders/README.md file](shaders/README.md).

The OBS plugin used for taking screenshots is a separate component, currently located [here](https://github.com/colinator27/dust-manipulator-plugin).

