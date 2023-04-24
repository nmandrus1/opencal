# OpenCal

OpenCal is a project that aims to provide collaborative calendars for individuals, teams, and organizations. With OpenCal, users can easily create, share, and manage calendars with colleagues, friends, and family members.

The platform promises to simplify the process of scheduling meetings, appointments, and events by allowing users to view each other's schedules and availability in real-time. It also features a range of tools that enable users to customize their calendars, set reminders, and receive notifications.

OpenCal is designed to be user-friendly, intuitive, and accessible from any device, including smartphones, tablets, and desktop computers. The platform ensures fast and secure data transfer, and there are future plans to integrates with popular productivity apps such as Google Calendar.

Overall, OpenCal is an innovative solution for individuals and organizations looking to streamline their scheduling processes and improve collaboration. With its collaborative calendars, users can save time, increase productivity, and achieve better results.

# Table of Contents

<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#development-stack">Development Stack</a>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#development">Development</a></li>
      </ul>
    </li>
    <li><a href="#roadmap">Roadmap</a></li>
    <li><a href="#contributions">Contributions</a></li>
    <li><a href="#resources">Resources</a></li>
  </ol>
</details>

## Development Stack

![Rust](https://img.shields.io/badge/Rust-000000?style=for-the-badge&logo=rust&logoColor=white)

## Getting Started

### Prerequisites

1. Install [Rust](https://doc.rust-lang.org/book/ch01-01-installation.html)
2. Install [Sycamore](https://sycamore-rs.netlify.app/docs/getting_started/installation)
3. Clone the project

```sh
git clone https://github.com/nmandrus1/opencal
```

5. Move to the frontend, install the needed dependencies and run the program

```sh
cd frontend && trunk serve
```

### Development

```bash
## Backend
# build 
cargo build

# run after build
./target/debug/opencal

# build && run
cargo run

## frontend
trunk serve
```

## Contributions

1. Create a branch

```sh
git checkout -b branch-name
```

2. Switch to the recently created branch

```sh
git checkout branch-name
```

3. Change the files that are needed
4. Add your changes to staging

```sh
git add .
```

5. Commit your files to staging

```sh
git commit -m "commit message"
```

6. Connect your branch to the repo and push up

```sh
git push -u origin branch-name
```

7. Update your local repository with current remote repository

```sh
git pull
```
## Roadmap

To Be Decided

## Resources

- [Rust Documentation](https://doc.rust-lang.org/book/ch01-01-installation.html)
- [Sycamore Documentation](https://sycamore-rs.netlify.app/docs/getting_started/hello_world)
- [Git Tutorial](https://www.atlassian.com/git/tutorials)
- [Markdown Badges](https://dev.to/envoy_/150-badges-for-github-pnk)
