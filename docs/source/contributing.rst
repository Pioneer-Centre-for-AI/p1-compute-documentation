Contributing
############
To get viewing and contribution privileges to this documentation contact one of the GitHub organization owners or the compute coordinator. Once you have been added to the GitHub you just have to access one of the links to the documentation in the ReadMe of the repository. This will result in a prompt from ReadTheDocs asking for access to your GitHub. This will allow for ReadTheDocs to use your GitHub login as authentication.

About
=====
This documentation resource is meant to support sharing of information within P1. The heterogeneity of P1 can make it hard to communicate and retain information. Slack is simply not the channel and this resource is meant to make it easier to be a researcher at P1.
There are two presentable versions of the documentation. One is latest, which is tied to the main branch of the GitHub repository. The other version is the stable version which is tied to a specific branch and updated occassionally.

Setup Contribution
==================
Contributions can be performed one of two ways. Directly on GitHub or through git interactions.

GitHub Workflow
===============
To contribute on GitHub create a new branch from the branching menu and name it after the feature you want to create. 

.. image:: ./docs/images/contributing/github_contributing_01.png
  :width: 400
  :alt: Get on the right branch or create a new one

Write your features with the edit button IMAGE HERE. Once you are done writing your contributions, verify that it builds on ReadTheDocs LINK HERE. Now it is time to merge all the changes that might have happened since you branched out into your current branch. This is to make sure that merging your newly created features don't create conflicts when merging your branch back into the main branch. Once you have also confirmed that your branch, having the main branch merged into it, builds you are ready to merge your branch back into the main branch. Once again make sure the branch (now it should be the latest branch) builds.

Git Workflow
============
Intro to contributing through git.


Tips and Tricks
===============
If the documents are built, but you don't see an update of the relevant page, highlight the URL at the top of the doc and delete the url until it is just latest and press enter. This is likely to show your update.

.. list-table:: Useful Resources
   :widths: 60
   :header-rows: 1

   * - Link
   * - `ReadTheDocs guide <https://docs.readthedocs.io/en/stable/>`_
   * - `Git branch and merge guide <https://docs.github.com/en/repositories/configuring-branches-and-merges-in-your-repository/defining-the-mergeability-of-pull-requests/managing-a-branch-protection-rule>`_
   * - `RST guide <https://sublime-and-sphinx-guide.readthedocs.io/en/latest/index.html>`_
