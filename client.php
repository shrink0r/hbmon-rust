<?php

$socket = socket_create(AF_INET, SOCK_STREAM, SOL_TCP);
if (socket_connect($socket, "127.0.0.1", 8888)) {
    $data =  [ 'foo' => 'bar', 'bar' => [ 'more' => 'data' ]];
    $msg = json_encode($data);
    socket_send($socket, $msg, strlen($msg), MSG_EOF);
}

socket_close($socket);
