# Contributing Guidelines

Firstly, thank you for wanting to contribute to this project! If you're
unsure on how to start contributing, please refer to the GitHub
documentation on [opening a pull request with
forks](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/proposing-changes-to-your-work-with-pull-requests/creating-a-pull-request-from-a-fork).

> There are plans to create a
> [mdBook](https://rust-lang.github.io/mdBook/) that explains the ideas
> and theory behind quantr in more detail. The maintainers' advice is to
> wait for the completion of this book before contributing.

### How to contribute 

Quantr uses the [Gitflow
strategy](https://www.atlassian.com/git/tutorials/comparing-workflows/gitflow-workflow),
for version control with git. Below is the general outline of this 
strategy, 

- fork this repository;
- from the newly forked repository, create a new branch from the `dev`
  branch and prefix it's name with feat- or fix-, if adding a feature or
  fixing a bug respectively;
- implement your changes in your newly created branch with a series of
  [descriptive commits](#what-is-a-meaningful-commit); 
- when finished with implementing your changes, make sure to add your
  name to the list of contributers in 
  [CONTRIBUTERS.md](CONTRIBUTERS.md);
- merge the `dev` branch into your created branch (to make sure it's
  up-to-date with latest changes) and resolve any conflicts; then
- open a pull request to merge into this repositories `dev` branch,
  filling out the template provided.

Before submitting your pull request, make sure that your branch:

- passes all tests by running `cargo test`;
- is formatted with `cargo fmt`;
- has additional unit testing to test your new code (if applicable); and
- has [signed
  off](#do-i-need-to-sign-off-with-my-name-and-email-address) on all
  commits with your name and email, for example `Emmy Noether
  <emmy.noether@rings.com>`. Your name and email should be on its own
  line and placed at the end of the commit. By doing so, you are
  agreeing to the terms and conditions outlined in the Developer
  Certificate of Origin which can be found in
  [COPYRIGHT.txt](COPYRIGHT.txt). 

When a pull request is opened, a template should appear that shows the
above as checklist (that should be ticked off), in addition to questions
that the contributor should answer

## What is a descriptive commit? 

This is only specified to help maintain a well-documented repository,
and help the maintainers review the pull request. 

What the maintainers would appreciate are commits with:

- one sentence titles, and less than 50 characters;
- a space between title and paragraph; and
- a paragraph hard-wrapped at 72 characters explaining the reasons for
  the commit.

Each commit should give justification to the changes and the edits that
the commit induces. These justifications should **not** be what and
where (as GitHub can clearly answer these), but help the reviewer in
understanding how the commit achieves the goal of the overall pull
request. 

### Do I need to sign off with my name and email address? 

Yes. By doing so, you are agreeing to the Developer Certificate of 
Origin terms and conditions as outlined in 
[COPYRIGHT.txt](COPYRIGHT.txt).  

GitHub verified signatures will be reserved for the maintainers; so any
contributions that aren't from maintainers will not be verified, but
nevertheless very welcome!

### Why all the guidelines? 

Even though these are only 'guidelines' and are not strict rules to
follow, we hope in doing so will ease the reviewing of pull requests for 
the maintainers; and more importantly create a well-documented 
repository.
