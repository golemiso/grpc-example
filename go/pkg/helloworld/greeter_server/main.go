package main

import (
	"context"
	"fmt"
	"io"
	"log"
	"net"
	"time"

	pb "github.com/golemiso/grpc_example/go/proto/helloworld"
	"google.golang.org/grpc"
)

const (
	port = ":50051"
)

type server struct {
	pb.UnimplementedGreeterServer
}

func (s *server) SayHello(ctx context.Context, in *pb.HelloRequest) (*pb.HelloReply, error) {
	log.Printf("Received: %v", in.GetName())
	return &pb.HelloReply{Message: "Hello " + in.GetName()}, nil
}

func (s *server) ServerStreaming(in *pb.HelloRequest, stream pb.Greeter_ServerStreamingServer) error {
	log.Printf("Received: %v", in.GetName())

	for i := 0; i < 5; i++ {
		if err := stream.Send(&pb.HelloReply{
			Message: "Hello " + in.GetName(),
		}); err != nil {
			return err
		}
		time.Sleep(time.Second * 1)
	}

	return nil
}

func (s *server) ClientStreaming(stream pb.Greeter_ClientStreamingServer) error {
	names := []string{}
	for {
		in, err := stream.Recv()
		log.Printf("Received: %v", in.GetName())
		if err == io.EOF {
			message := fmt.Sprintf("Hello %v", names)
			return stream.SendAndClose(&pb.HelloReply{
				Message: message,
			})
		}
		if err != nil {
			return err
		}

		names = append(names, in.GetName())
	}
}

func (s *server) BidirectionalStreaming(stream pb.Greeter_BidirectionalStreamingServer) error {
	names := []string{}
	for {
		in, err := stream.Recv()
		log.Printf("Received: %v", in.GetName())
		if err == io.EOF {
			return nil
		}
		if err != nil {
			return err
		}
		message := fmt.Sprintf("Hello %v", names)
		err = stream.Send(&pb.HelloReply{
			Message: message,
		})
		if err != nil {
			return err
		}

		names = append(names, in.GetName())
	}
}

func main() {
	lis, err := net.Listen("tcp", port)
	if err != nil {
		log.Fatalf("failed to listen: %v", err)
	}
	s := grpc.NewServer()
	pb.RegisterGreeterServer(s, &server{})
	log.Printf("server listening at %v", lis.Addr())
	if err := s.Serve(lis); err != nil {
		log.Fatalf("failed to serve: %v", err)
	}
}
