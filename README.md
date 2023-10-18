# gus

gus will be a general solution for setting up a REST-ful webserver with a database backend.

The idea is that the defined models (json) will be parsed and used by the model-layer (model controller view). The controller, the actual webserver, will serve the REST-API for them.<br>
There will be a fully functional view layer as well, but first I need to enhance my frontend skills.

In the end, gus would be a nice and simple binary tool for private or whatever use. Of course I will try to make it as customizable and convenient as possible.

# Testing 
During development, until gus is ready for release, I serve the current release binaries of the latest commit on the separate branch `debug-binaries` for testing on Linux, Mac and Windows.<br>
So feel free to try the current state.
 - [Linux](https://www.github.com/qwerdenkerXD/gus/raw/debug-binaries/gus-linux)
 - [MacOS](https://www.github.com/qwerdenkerXD/gus/raw/debug-binaries/gus-mac)
 - [Windows](https://www.github.com/qwerdenkerXD/gus/raw/debug-binaries/gus-win.exe)

Of course you can compile them by yourself for your OS.
<br>Simply run:<br>
`cargo install --git https://github.com/qwerdenkerXD/gus`

I also host an example webserver with the movie model from testing on [Replit](https://gus.qwerdenkerxd.repl.co/).

Testing Hint: The REST-API has currently no model name inflection implemented, so to test the current CRUD-one functions, access them via e.g. endpoint ``/api/rest/<model-name>/<id>`` or for POST without the ID.