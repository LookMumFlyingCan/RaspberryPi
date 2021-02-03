cp /boot/initial/fishconfig /home/dietpi/.config/fish/config.fish

sudo su
exit

fish
exit

sudo cp /boot/initial/fishconfig /root/.config/fish/config.fish
mkdir /home/dietpi/.config/fish/functions
sudo mkdir /root/.config/fish/functions

cp /boot/userprompt /home/dietpi/.config/fish/functions/fish_prompt.fish
sudo cp /boot/rootprompt /etc/systemd/system/dietpi-boot.service

git clone https://github.com/chesty/overlayroot chesty

cd chesty
sudo ./install.sh

cd /usr/bin/
sudo wget https://raw.githubusercontent.com/chesty/overlayroot/master/rootwork
sudo chmod a+x rootwork

reboot
