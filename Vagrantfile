# -*- mode: ruby -*-
# vi: set ft=ruby :

Vagrant.configure("2") do |config|
    config.vm.box = "ubuntu/trusty64"

    config.vm.provider "virtualbox" do |vb|
        vb.memory = 128
        vb.cpus = 1
    end

    config.vm.provision "shell", inline: "echo Hello World"

    now = Time.now.to_i

    binary = "/vagrant/target/debug/dht -c /vagrant/config.ini"
    output = "/home/vagrant/dht-#{now}.out"
    error = "/home/vagrant/dht-#{now}.out"

    (1..10).each do |i|
        config.vm.define "peer-#{i}" do |peer|
            peer.vm.network "private_network", ip: "192.168.42.#{10 + i}"
            peer.vm.provision "shell", inline: "echo I\\'m peer #{i}"

            cmd = "#{binary} -b 192.168.42.#{9 + i}:31415" if i > 1

            peer.vm.provision "shell", inline: "#{cmd} > #{output} 2> #{error} &",
                name: "distributed hash table", run: "always", privileged: false
        end
    end
end
