/// <reference path="../pb_data/types.d.ts" />
migrate((app) => {
  const collection = app.findCollectionByNameOrId("pbc_2822314327")

  // update collection data
  unmarshal({
    "viewRule": "author.id=@request.auth.id || public=TRUE"
  }, collection)

  return app.save(collection)
}, (app) => {
  const collection = app.findCollectionByNameOrId("pbc_2822314327")

  // update collection data
  unmarshal({
    "viewRule": ""
  }, collection)

  return app.save(collection)
})
