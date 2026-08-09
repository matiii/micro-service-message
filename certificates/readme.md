### CA key + self-signed root cert
- openssl ecparam -genkey -name prime256v1 -out ca/ca-key.pem
- openssl req -new -x509 -key ca/ca-key.pem -days 3650 -out ca/ca-cert.pem \
-subj "/CN=ServiceMessage CA"

### Server key + CSR + cert signed by CA
- openssl ecparam -genkey -name prime256v1 -out server/server-key.pem
- openssl req -new -key server/server-key.pem -out server/server.csr -subj "/CN=service-message-broker"
- openssl x509 -req -in server/server.csr -CA ca/ca-cert.pem -CAkey ca/ca-key.pem \
-CAcreateserial -out server/server-cert.pem -days 825 \
-extfile <(echo "subjectAltName=DNS:broker.internal,DNS:localhost")

### Client key + CSR + cert signed by CA
- openssl ecparam -genkey -name prime256v1 -out client/client-key.pem
- openssl req -new -key client/client-key.pem -out client/client.csr -subj "/CN=some-namespace-client"
- openssl x509 -req -in client/client.csr -CA ca/ca-cert.pem -CAkey ca/ca-key.pem \
-CAcreateserial -out client/client-cert.pem -days 825