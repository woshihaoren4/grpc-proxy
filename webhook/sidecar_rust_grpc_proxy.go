package main

import (
	"fmt"
	"github.com/gin-gonic/gin"
	"github.com/sirupsen/logrus"
	"io/ioutil"
	v1 "k8s.io/api/admission/v1"
	corev1 "k8s.io/api/core/v1"
	metav1 "k8s.io/apimachinery/pkg/apis/meta/v1"
	"k8s.io/apimachinery/pkg/runtime"
	"k8s.io/apimachinery/pkg/runtime/serializer"
	"k8s.io/apimachinery/pkg/types"
	"net/http"
)

var (
	sidecarName      = "rust-grpc-proxy-sidecar"
	lable            = "rustGrpcProxyEnable"
	podsSidecarPatch = `[{"op":"add", "path":"/spec/containers/-","value":{"env":[{"name":"RUST_GRPC_PROXY_ADDR","value": "%s"}],"image":"%s","name":"%s","resources":{}}}]`
	image            = "wdshihaoren/rust-grpc-proxy:latest"
	runtimeScheme    = runtime.NewScheme()
	codecs           = serializer.NewCodecFactory(runtimeScheme)
	deserializer     = codecs.UniversalDeserializer()
)

func SidecarRustGrpcProxy(c *gin.Context) {
	body, err := ioutil.ReadAll(c.Request.Body)
	if err != nil {
		ResponseError(c, "read body error:%v", err)
		return
	}
	logrus.Infoln("request--->", string(body))
	contentType := c.GetHeader("Content-Type")
	if contentType != "application/json" {
		c.String(http.StatusUnsupportedMediaType, "md not real")
		return
	}
	review := new(v1.AdmissionReview)
	if _, _, err = deserializer.Decode(body, nil, review); err != nil {
		ResponseError(c, "Unmarshal AdmissionReview error:%v", err)
		return
	}
	podResource := metav1.GroupVersionResource{Group: "", Version: "v1", Resource: "pods"}
	if review.Request.Resource != podResource {
		ResponseError(c, "expect resource to be %s", podResource)
		return
	}
	pod := new(corev1.Pod)
	if _, _, err = deserializer.Decode(review.Request.Object.Raw, nil, pod); err != nil {
		ResponseError(c, "deserializer pod error:%v", err)
		return
	}
	for _, i := range pod.Spec.Containers {
		if i.Name == sidecarName {
			ResponseAllow(c, review.Request.UID)
			return
		}
	}
	val, ok := pod.Labels[lable]
	if !ok {
		ResponseAllow(c, review.Request.UID)
		return
	}
	resp := new(v1.AdmissionResponse)
	resp.UID = review.Request.UID
	resp.Allowed = true
	resp.Patch = []byte(fmt.Sprintf(podsSidecarPatch, "127.0.0.1:"+val, image, sidecarName))
	pt := v1.PatchTypeJSONPatch
	resp.PatchType = &pt

	respReview := &v1.AdmissionReview{
		Request:  nil,
		Response: resp,
	}

	respReview.Kind = "AdmissionReview"
	respReview.APIVersion = "admission.k8s.io/v1"
	c.JSON(http.StatusOK, respReview)
}

func ResponseError(c *gin.Context, format string, values ...any) {
	logrus.Infoln("error:", fmt.Sprintf(format, values...))
	c.String(http.StatusBadRequest, fmt.Sprintf(format, values...))
}
func ResponseAllow(c *gin.Context, uid types.UID) {
	ar := &v1.AdmissionReview{
		Response: &v1.AdmissionResponse{
			UID:     uid,
			Allowed: true,
		},
	}
	ar.Kind = "AdmissionReview"
	ar.APIVersion = "admission.k8s.io/v1"
	c.JSON(http.StatusOK, ar)
}
