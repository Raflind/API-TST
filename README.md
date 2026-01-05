# Cara Penggunaan

Seluruh API dapat diakses melalui domain bebasdah.theokaitou.my.id

dengan Endpoint berikut:

# Endpoint
1. /movies
- Method: GET
- Description: Mengambil daftar film
- Parameter: -
Contoh Command : 
```bash
curl -X GET http://bebasdah.theokaitou.my.id/movies


2. /chat
- Method: POST
- Description: Mengirim pesan ke chatroom
- Request Body: 
    - movie_id: ID dari film
    - user_id: ID pengguna
    - chat: pesan yang dikirim

Contoh Command : 
```bash
curl -X POST http://bebasdah.theokaitou.my.id/chat \
  -H "Content-Type: application/json" \
  -d '{
    "movie_id": 1,
    "user_id": 1,
    "chat": "filem bagus!"
  }'
