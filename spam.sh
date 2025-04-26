#!/bin/bash

# Устройство
DEVICE="/dev/ttyUSB1"

while true; do
    # Отправляем байты на устройство
    echo -n -e "abcdefgh" > $DEVICE
    # Ждем 0.5 секунды
    sleep 0.5
done
