/// <reference path="../pb_data/types.d.ts" />
migrate((app) => {
  const collection = app.findCollectionByNameOrId("pbc_2822314327")

  // update collection data
  unmarshal({
    "deleteRule": "author.id=@request.auth.id"
  }, collection)

  return app.save(collection)
}, (app) => {
  const collection = app.findCollectionByNameOrId("pbc_2822314327")

  // update collection data
  unmarshal({
    "deleteRule": ""
  }, collection)

  return app.save(collection)
})
