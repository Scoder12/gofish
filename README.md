# gofish

Building FlatBuffers:

```sh
cd api
( cat ../schema.fbs; echo 'root_type CmsgTable;' ) > ./schema_server.fbs; flatc --rust -o src/ ./schema_server.fbs
```

```sh
cd frontend
( cat ../schema.fbs; echo 'root_type SmsgTable;' ) > ./schema_client.fbs; flatc --ts -o src/ ./schema_client.fbs
```
