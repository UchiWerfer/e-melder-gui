# E-Melder-GUI
## What is E-Melder-GUI
E-Melder-GUI is an alternative application to sign up athletes for competitions, if the organizer of an event uses the E-Melder software by DATASERVICE Software.
It is designed to have a better UX than the official application and to be, different from the official application, FOSS.

## Installation
1. Download the latest release for your platform (Windows or Linux) from the releases tab here on Github.
2. Run the contained executable (for Windows the .exe-file).
3. (Optionally) Configure the dark-mode or language (currently supported are German and English) to your liking, when you deviate from the supported languages, make sure the correct language-file is in the correct spot (view the code if unclear where that is)

## Finding the registering-files
If you have set the option being the folder for the registering-files, you will find them in the specified folder. Otherwise you will find them by going into your home-folder and searching the "e-melder" folder in there. Here you will find the files.

## Updating
1. Download the latest release for your platform (Windows or Linux) from the releases tab here on Github.
2. Run the contained executable (for Windows the .exe-file).
3. If encounter a message like about.no_network in places you would expect a translation into your language and you are using one of the supported languages, delete the translations file for your installation (use "Where are the translation files stored" to find the translations file)

## Where are the translation files stored
If you are running Windows, hit Windows+R and enter "%APPDATA%" and press the Enter-key, locate the "e-melder" folder there, and go into the "lang" folder to find your translations.
If an error occurres after pressing the Enter-key, go into your home-folder and follow the steps described above after pressing the Enter-key
If you are running Linux, check your $XDG_CONFIG_HOME environment-variable, if it is set you will find the translations in $XDG_HOME_CONFIG/e-melder/lang, otherwise in ~/.config/e-melder/lang.
