package main

import (
	"crypto/tls"
	"flag"
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
	"net/http"
)

var (
	certFile     string
	keyFile      string
	port         int
	sidecarImage string
)

func initFlag() {
	flag.IntVar(&port, "port", 443, "Webhook server port.")
	flag.StringVar(&certFile, "tlsCertFile", "./certs/cert.pem", "File containing the x509 Certificate for HTTPS.")
	flag.StringVar(&keyFile, "tlsKeyFile", "./certs/key.pem", "File containing the x509 private key to --tlsCertFile.")
	flag.Parse()
}

func configTLS(cert, key string) *tls.Config {
	sCert, err := tls.LoadX509KeyPair(cert, key)
	if err != nil {
		panic(err)
	}
	return &tls.Config{
		Certificates: []tls.Certificate{sCert},
		// TODO: uses mutual tls after we agree on what cert the apiserver should use.
		// ClientAuth:   tls.RequireAndVerifyClientCert,
	}
}

func main() {
	initFlag()
	gin.SetMode(gin.ReleaseMode)

	app := gin.Default()

	app.POST("/sidecar/rust-grpc-proxy", SidecarRustGrpcProxy)

	server := &http.Server{
		Addr:      fmt.Sprintf(":%d", port),
		TLSConfig: configTLS(certFile, keyFile),
		Handler:   app,
	}
	logrus.Infoln("webhook server running")
	err := server.ListenAndServeTLS("", "")
	if err != nil {
		panic(err)
	}
}
