# carbon-app

Read and search Materials from the EC3 API in a GUI 

The app consists of a a main App written in Rust with Egui and a library called EC3 API that handles querying the EC3API.

## Usage 
To use the app you must have your own ec3 account, create a token and set it into the environment with the key  `API_KEY`.

For example:



```
# .env in the root folder
API_KEY="..."
```

## Screenshots

![Screenshot from 2023-07-26 19-40-27](https://github.com/andrsbtrg/carbon-app/assets/63083862/60e30e08-bb30-4463-ad84-67c0bd5d24c5)

## TODO!
- Visualization of the GWP per element and per category
- Indexing and searching per keyword. Basically make it possible to find a related material with some keywords.

