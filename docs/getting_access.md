Before accessing any of the P1 HPCs you must first [register to become a member of P1](https://www.aicentre.dk/affiliation). You do this by filling out this [form](https://da.surveymonkey.com/r/P1Affiliation). Once you've correctly been added to [the peoples list](https://www.aicentre.dk/people) you can proceed with accessing the cluster.

Uncertain which cluster to chose? You can reach out to the [Compute Coordinator](mailto:bstja@dtu.dk) to get a project assessment on which HPC to chose.

## Getting access: P1 DTU HPC

This cluster is hosted at DTU and the next step is therefore to get a DTU account, so that the DTU compute environment can be accessed. This can be obtained by filling out this form <https://forms.office.com/e/DG5qCfs6Wm> which will be sent to Henning Christiansen, head of DTUs compute center. Only signup using an official university email address is accepted. He will create an account for you and contact you with the details.

- For technical issues about the P1 DTU HPC, like broken or missing packages, contact <support@hpc.dtu.dk>

### Accessing the cluster

To access the cluster outside of DTUs network please refer to the guide in the following section.

The compute cluster is accessible at `login9.hpc.dtu.dk` via SSH, where the user will log into their home directory. Note hat the home directories have limited storage (50gb), but more storage is available at `/dtu/p1/`.

We have 1 interactive node that you can use for installing packages and try-runs of your experiments. Please do not run any heavy jobs on the login node, and use [batch jobs](https://www.hpc.dtu.dk/?page_id=1416) for your experiments.

#### Accessing the cluster outside of DTU

This section describes one possible option to access the cluster from outside of DTUs network using a Linux machine. The guide assumes that you have obtained a DTU account beforehand.

#### First access steps

1. Download Cisco AnyConnect VPN client
1. Go to https://dtubasen.dtu.dk and choose sign-in via Azure multi-factor auth. using your full DTU username
   (including `@dtu.dk`)
1. Set up a new multi-factor auth to DTU e.g via Authenticator app.
1. Establish connection to vpn.dtu.dk in the AnyConnect client.
1. Log in to the cluster via SSH at `login9.hpc.dtu.dk` using your DTU credentials (username without `@dtu.dk`).
1. Close connection to the cluster.

You can choose to keep using VPN to access the cluster or use SSH keys + passwords instead. For the latter please
follow the steps below.

#### Access with SSH keys + passwords

1. Ensure that you have completed the steps described above to establish access via VPN.
1. Generate an SSH key using ssh-keygen.
1. Copy the key to the server e.g. using: `ssh-copy-id -i ~/.ssh/keyname username@login9.hpc.dtu.dk`
1. Log in to the cluster: `ssh username@login9.hpc.dtu.dk`

For more technical information please refer to the
[P1 compute cluster documentation](https://www.hpc.dtu.dk/?page_id=5028) at DTU DCC.

## Getting access: P1 NGC HPC

The P1 NGC HPC is hosted at the National Genome Centre and is GDPR compliant. To get access, you need to:

1. Complete the P1 membership registration as described at the top of this document
1. Contact the [Compute Coordinator](mailto:bstja@dtu.dk) to request access to the NGC HPC as we'll need to ensure a proper data processing agreement is in place.

### Accessing the cluster

The P1 NGC HPC is an air-gapped system. And you'll need to setup MFA and a client for accessing the remote VM entrypoint. More information about this is shared by NGC support once the user has been registered on the system.

## Getting access: P1 Gefion HPC

The P1 Gefion HPC is a new addition to P1's compute resources. To get access:

1. Complete the P1 membership registration as described at the top of this document
1. Contact the [Compute Coordinator](mailto:bstja@dtu.dk) to request access to the Gefion HPC

### Accessing the cluster

To be determined... Guidelines will follow.
