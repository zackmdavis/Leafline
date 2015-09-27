# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure(2) do |config|
  config.vm.box = "ubuntu/trusty64"
  config.vm.network "forwarded_port", guest: 2882, host: 2851  # Kasparov
  config.vm.network :private_network, ip: "192.168.8.8"
  config.vm.provision "ansible" do |ansible|
    ansible.playbook = "provisioning/leafline.yml"
  end
end
