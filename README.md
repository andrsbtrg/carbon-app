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

![image](https://github.com/andrsbtrg/carbon-app/assets/63083862/7c3e8791-b10c-4f1b-8f1e-d8752b3cf2ec)
  
![image](https://github.com/andrsbtrg/carbon-app/assets/63083862/bcbb8500-2886-41ce-87e9-4874a609cebd)

![image](https://github.com/andrsbtrg/carbon-app/assets/63083862/07a26486-619a-4b8b-9393-0d45f11193b6)

![image](https://github.com/andrsbtrg/carbon-app/assets/63083862/5a3b2900-8405-4da9-bdc2-5562fee4fbe8)


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
- [x] Add the material id from the api, to avoid clashes between materials
- [x] Add a list of categories to make searching easier
- [ ] Add a graph of material categories to visualize 
- [ ] Visualize GWP averages per category on graph view


