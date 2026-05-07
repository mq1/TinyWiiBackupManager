ARG ARCH

FROM $ARCH/almalinux:8

RUN yum install -y openssl-devel
