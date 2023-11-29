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

![output](https://github.com/andrsbtrg/carbon-app/assets/63083862/4b18a75f-d9e1-4dbf-a988-bfa1e77dbe27)

## EC3 API

### Material filter:
The app implements its own 'Material Filter' serializer to query from the EC3 API.


Sample request:

```json
{"pragma":[{"name":"eMF","args":["2.0/1"]},{"name":"lcia","args":["EF 3.0"]}],"category":"Concrete","filter":[{"field":"jurisdiction","op":"in","arg":["150"]},{"field":"epd_types","op":"in","arg":["Product EPDs","Industry EPDs"]}]}
```


Sample response:

```
'!EC3 search("Concrete") WHERE \n jurisdiction: IN("150") AND\n epd_types: IN("Product EPDs", "Industry EPDs") \n!pragma eMF("2.0/1"), lcia("EF 3.0")'
```

## TODO!
- [x] Hot reloading. Implemented following the example of [rust-hot-reloading](https://github.com/irh/rust-hot-reloading/tree/main)
- [x] Visualization of the GWP per element and per category.
- [ ] Indexing and searching per keyword. Basically make it possible to find a related material with some keywords.
- [x] Loading materials via API without blocking UI.
- [x] Searching new materials via API.
- [ ] Add the material id from the api, to avoid clashes between materials
- [ ] Add a list of categories to make searching easier
- [ ] Add a graph of material categories to visualize 


