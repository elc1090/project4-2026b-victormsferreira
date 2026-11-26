/// <reference path="../pb_data/types.d.ts" />
migrate((app) => {
  const collection = app.findCollectionByNameOrId("pbc_2822314327")

  // add field
  collection.fields.addAt(5, new Field({
    "help": "",
    "hidden": false,
    "id": "file3277268710",
    "maxSelect": 0,
    "maxSize": 0,
    "mimeTypes": [
      "image/png"
    ],
    "name": "thumbnail",
    "presentable": false,
    "protected": false,
    "required": false,
    "system": false,
    "thumbs": [
      "320x240"
    ],
    "type": "file"
  }))

  return app.save(collection)
}, (app) => {
  const collection = app.findCollectionByNameOrId("pbc_2822314327")

  // remove field
  collection.fields.removeById("file3277268710")

  return app.save(collection)
})
