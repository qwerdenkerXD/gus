# gus

gus will be a general solution for setting up a webserver with a database backend.

The setup is really easy designed. You just define your data models in JSON files and run the server. That's it.<br>
To make it more convenient and to save time of reading unnecessarily long manuals to use the tool, I implemented a CLI dialogue to configure more complex structures such as these data models by running therefore e.g. `gus create-model`.
Interacting with these wizards is intuitive and guarantees correct definitions and helps to learn fast to create the respective files without the need of any guidance.

gus will serve two APIs to interact with the databases, a REST API and GraphQL. As frontend I'll create a React App to have an intuitive GUI that will be platform independent, so also a good thing for mobile devices...or better, that's the plan...
For the GraphQL API I provide the Graph*i*QL interface as well. To make gus more customizable, I think about adding the feature to define an own frontend. So if you aren't happy with my React skills that are currently very weak, you'll have the opportunity to develop you own app and serve it instead of mine. Same thing would be possible for Graph*i*QL if you have an implementation that suits you better.

As possible databases I currently have only a JSON storage implemented, just to develop the webserver. But if gus is ready for release, I think about adding much more storage types, such as MongoDB, SQLite, Neo4j and others, a distributed setup including. Because I use the model-controller-view paradigm to develop gus, this won't be very complicated since I just have to concentrate on how to get records into the respective database.
I'm also confident that migrations, associations and constraints won't be a big problem, so gus could be a very powerful tool in the end.

# Testing 
During development, until gus is ready for release, I serve the current release binaries of the latest commit on the separate branch `debug-binaries` for testing on Linux, Mac and Windows.<br>
So feel free to try the current state.
 - [Linux](https://www.github.com/qwerdenkerXD/gus/raw/debug-binaries/gus-linux)
 - [MacOS](https://www.github.com/qwerdenkerXD/gus/raw/debug-binaries/gus-mac)
 - [Windows](https://www.github.com/qwerdenkerXD/gus/raw/debug-binaries/gus-win.exe)

Of course you can compile them by yourself for your OS.
<br>Simply run:<br>
`cargo install --locked --git https://github.com/qwerdenkerXD/gus`

I also host an example webserver with the movie model from testing on [Replit](https://gus.qwerdenkerxd.repl.co/api/graphql).

The API endpoints are located at `/api/rest/...` and `/api/graphql`.
