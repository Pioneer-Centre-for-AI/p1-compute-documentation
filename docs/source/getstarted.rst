Get started
###########

The mission of the P1 HPC is to support research from affiliates. So to get
access to the cluster, one should first 
`register to be a member of P1 <https://www.aicentre.dk/affiliation>`_. The next 
step is to get a DTU account, so that the DTU compute environment can be accessed. 
This can be obtained on: TODO.com. Please note that the P1 HPC is only accessible 
through the DTU network, so if you are not within the network you have to be on the 
`VPN <https://www.hpc.dtu.dk/?page_id=4317>`_.

The compute cluster can be accessed through ssh on login9.hpc.dtu.dk, where the
user will log into their own home directory. Note that the home directories
have limited storage (50gb), and there is much more storage available on
`/dtu/p1/`.

We have 1 interactive nodes, which you can use for installing packages and 
try-runs of your experiments. Please do not run any heavy jobs on the login
node, and use `batch jobs <https://www.hpc.dtu.dk/?page_id=1416>`_ for your
experiments.
 
When having used support from DCC (the DTU HPC center administrating our DTU
cluster) please cite the following reference:

.. code-block:: RST

  http://doi.org/10.48714/DTU.HPC.0001

Accessing the cluster outside of DTU
************************************
To access the cluster from outside of DTU the section below describes one possible option for Linux machines.

**First access steps:**
1. Download Cisco AnyConnect VPN client.
2. Go to https://dtubasen.dtu.dk and choose sign-in via Azure multi-factor auth using your full DTU username (including @dtu.dk).
3. Set up a new multi-factor auth to DTU e.g via Authenticator app.
4. Establish connection to vpn.dtu.dk in the AnyConnect client.
5. Log-in to the cluster via SSH at login9.hpc.dtu.dk using your DTU credentials (username without @dtu.dk).
6. Close connection to the cluster.

You can choose to keep using VPN to access the cluster or use SSH keys + passwords instead. For the latter please follow the steps below.

**Access with SSH keys + passwords**
1. Ensure that you have completed the steps described above to establish access via VPN.
2. Generate a SSH key using ssh-keygen.
3. Copy the key to the server e.g. using: ssh-copy-id -i ~/.ssh/keyname username@login9.hpc.dtu.dk
4. Log-in to the cluster: ssh username@login9.hpc.dtu.dk


For more technical information please refer to the 
`P1 compute cluster documentation <https://www.hpc.dtu.dk/?page_id=5028>`_ at DTU DCC.

TODO, do we want to have our own reference?
