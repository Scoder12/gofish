# gofish

Building FlatBuffers:

```sh
cd api
( cat ../schema.fbs; echo 'root_type CMsgTable;' ) > ./schema_server.fbs; flatc --rust -o src/ ./schema_server.fbs
```

```sh
cd frontend
( cat ../schema.fbs; echo 'root_type SMsgTable;' ) > ./schema_client.fbs; flatc --ts -o src/ ./schema_client.fbs
```
