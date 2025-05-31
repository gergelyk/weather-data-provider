var ID = (curl -d "Hello World!" -X POST http://127.0.0.1:3000/pbproxy)
curl http://127.0.0.1:3000/pbproxy/$ID
