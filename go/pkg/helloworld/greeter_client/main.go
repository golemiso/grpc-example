package main

import (
	"context"
	"errors"
	"io"
	"log"
	"os"
	"time"

	pb "github.com/golemiso/grpc_example/go/proto/helloworld"
	"golang.org/x/sync/errgroup"
	"google.golang.org/grpc"
)

const (
	address     = "[::1]:50051"
	defaultName = "Sekai"
)

type client struct {
	cli pb.GreeterClient
}

func (c *client) SayHello() {
	name := defaultName
	resp, err := c.cli.SayHello(context.Background(), &pb.HelloRequest{Name: name})
	if err != nil {
		log.Fatalf("could not greet: %v", err)
	}
	log.Printf("Greeting: %s", resp.GetMessage())
}

func (c *client) ServerStreaming() {
	name := defaultName
	stream, err := c.cli.ServerStreaming(context.Background(), &pb.HelloRequest{Name: name})
	if err != nil {
		log.Fatalf("could not greet: %v", err)
	}
	defer stream.CloseSend()
	for {
		resp, err := stream.Recv()
		if errors.Is(err, io.EOF) {
			return
		}
		if err != nil {
			log.Fatalf("could not greet: %v", err)
		}
		log.Printf("Greeting: %s", resp.GetMessage())
	}
}

func (c *client) ClientStreaming() {
	stream, err := c.cli.ClientStreaming(context.Background())
	if err != nil {
		log.Fatalf("could not greet: %v", err)
	}
	values := []string{"1", "2", "3", "4", "5"}
	for _, value := range values {
		if err := stream.Send(&pb.HelloRequest{
			Name: value,
		}); err != nil {
			if err == io.EOF {
				break
			}
			log.Fatalf("could not greet: %v", err)
		}
		time.Sleep(time.Second * 1)
	}
	resp, err := stream.CloseAndRecv()
	if err != nil {
		log.Fatalf("could not greet: %v", err)
	}
	log.Printf("Greeting: %s", resp.GetMessage())
}

func (c *client) BidirectionalStreaming() {
	stream, err := c.cli.BidirectionalStreaming(context.Background())
	if err != nil {
		log.Fatalf("could not greet: %v", err)
	}

	var eg errgroup.Group
	eg.Go(func() error {
		values := []string{"1", "2", "3", "4", "5"}
		defer stream.CloseSend()
		for _, value := range values {
			if err := stream.Send(&pb.HelloRequest{
				Name: value,
			}); err != nil {
				if err == io.EOF {
					return nil
				}
				log.Fatalf("could not greet: %v", err)
			}
			time.Sleep(time.Second * 1)
		}
		return nil
	})
	eg.Go(func() error {
		for {
			resp, err := stream.Recv()
			if err == io.EOF {
				return nil
			}
			if err != nil {
				log.Fatalf("could not greet: %v", err)
			}
			log.Printf("Greeting: %s", resp.GetMessage())
		}
	})

	if err := eg.Wait(); err != nil {
		log.Fatalf("could not greet: %v", err)
	}
}

func main() {
	// Set up a connection to the server.
	conn, err := grpc.Dial(address, grpc.WithInsecure(), grpc.WithBlock())
	if err != nil {
		log.Fatalf("did not connect: %v", err)
	}
	defer conn.Close()
	c := &client{
		cli: pb.NewGreeterClient(conn),
	}

	switch os.Args[1] {
	case "1":
		c.SayHello()
	case "2":
		c.ServerStreaming()
	case "3":
		c.ClientStreaming()
	case "4":
		c.BidirectionalStreaming()
	default:
		log.Fatal("could not greet")
	}
}
