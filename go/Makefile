.PHONY: proto-gen
proto-gen:
	protoc --go_out=. --go_opt=paths=source_relative \
    --go-grpc_out=. --go-grpc_opt=paths=source_relative \
    proto/helloworld/helloworld.proto

.PHONY: run-server run-client
run-server:
	go run pkg/helloworld/greeter_server/main.go
run-client:
	go run pkg/helloworld/greeter_client/main.go
