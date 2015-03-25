<?php

$socket = socket_create(AF_INET, SOCK_STREAM, SOL_TCP);
if (socket_connect($socket, "127.0.0.1", 8888)) {
    $data =  [ 'foo' => 'bar', 'bar' => [ 'more' => 'data' ]];
    $msg = json_encode($data);
    $msg = str_pad(strlen($msg), 4, "0", STR_PAD_LEFT) . $msg;
    socket_write($socket, $msg, strlen($msg));
print_r($msg);
    sleep(3);

    $data =  [ 'bar' => 'foo', 'snafu' => [ 'fnord' => '42' ]];
    $msg = json_encode($data);
    $msg = str_pad(strlen($msg), 4, "0", STR_PAD_LEFT) . $msg;
    socket_write($socket, $msg, strlen($msg));
print_r($msg);
}

socket_close($socket);
