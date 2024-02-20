# Get started

The mission of the P1 HPC is to support research from affiliates. To get access to the cluster, one should first 
[register to be a member of P1](https://www.aicentre.dk/affiliation). Since the cluster is hosted at DTU, the next 
step is to get a DTU account, so that the DTU compute environment can be accessed. This can be obtained by sending a
email to head of DTUs compute center, Henning Christiansen, at <hench@dtu.dk>. He will manually create an account for
you and contact you with the details. To access the cluster outside of DTUs network please refer to the guide in the 
following section.

The compute cluster is accessible at `login9.hpc.dtu.dk` via SSH, where the user will log into their home directory. 
Note hat the home directories have limited storage (50gb), but more storage is available at `/dtu/p1/`.

We have 1 interactive node that you can use for installing packages and try-runs of your experiments. Please do not run 
any heavy jobs on the login node, and use `batch jobs <https://www.hpc.dtu.dk/?page_id=1416>`_ for your experiments.
 
When having used support from DCC (the DTU HPC center administrating our DTU cluster) please cite the following 
reference:

```{bib}
http://doi.org/10.48714/DTU.HPC.0001
```

## Accessing the cluster outside of DTU

This section describes one possible option to access the cluster from outside of DTUs network using a Linux machine. 
The guide assumes that you have obtained a DTU account beforehand.

## First access steps

1. Download Cisco AnyConnect VPN client
2. Go to https://dtubasen.dtu.dk and choose sign-in via Azure multi-factor auth. using your full DTU username 
    (including `@dtu.dk`)
3. Set up a new multi-factor auth to DTU e.g via Authenticator app.
4. Establish connection to vpn.dtu.dk in the AnyConnect client.
5. Log in to the cluster via SSH at `login9.hpc.dtu.dk` using your DTU credentials (username without `@dtu.dk`).
6. Close connection to the cluster.

You can choose to keep using VPN to access the cluster or use SSH keys + passwords instead. For the latter please 
follow the steps below.

## Access with SSH keys + passwords

1. Ensure that you have completed the steps described above to establish access via VPN.
2. Generate an SSH key using ssh-keygen.
3. Copy the key to the server e.g. using: `ssh-copy-id -i ~/.ssh/keyname username@login9.hpc.dtu.dk`
4. Log in to the cluster: `ssh username@login9.hpc.dtu.dk`

For more technical information please refer to the 
[P1 compute cluster documentation](https://www.hpc.dtu.dk/?page_id=5028) at DTU DCC.
