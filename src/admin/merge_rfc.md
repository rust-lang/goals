# Merging the RFC

Once the RFC is accepted, you need to take the following steps.

## Merge the RFC itself

Update the RFC and merge as normal

## Update the teams

Run `cargo rpg teams` to prepare adjustment to the teams repository. You will need to have a checkout of the teams repository somewhere; you pass the command the path to that repo and it will make changes. You can then commit the changes and prepare a PR. 

This will create a `project-goal-owners` team containing all the project goal owners. It will also add people to the rust-lang repository. This may trigger them to get invites if they are not already members of the org. You should encourage them to accept those invites. If they don't take these steps you won't be able to assign them to issues and they won't be able to author updates, etc.

## Create the milestone

Next you need to (manually) create a milestone on the goals repository with the appropriate name (e.g., `2025h1`). We usually create a paired meta milestone like `2025h1-meta` to track other tasks related to running the program.

## Create tracking issues

Finally, you can create the tracking issues. To do this, you run `cargo rpg issues`. Before doing so, make sure that the metadata for any goals that are continuing from the previous milestone already lists the appropriate tracking issue, otherwise the comment will create a duplicate issue.

You can run the command more than once, it tries to pick up from where it left off. It will adjust the state of all issues to match what is expected.

## Close old tracking issues

Finally, you can go to the previous milestone, find all remaining issues, and close them. These should correspond to goals from the previous session that were not continued into the current one.
