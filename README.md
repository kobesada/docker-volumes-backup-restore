# Backup and Restore Docker Volumes to Server via SSH

This program is designed to back up and restore Docker volumes and other directories, ensuring data integrity by
stopping containers during the process. Backups are stored on a remote server via SSH.
Written in Rust (street_credit++), the program runs inside a Docker container, making it easy to integrate into any
Docker Compose setup.

## Features

- **Backup Docker Volumes**: Automatically stops containers using specific volumes, archives them, and restarts the
  containers.
- **Customizable Backup Directory**: Backup any directories by mounting them to the `/backup` folder in Docker.
- **Scheduled Backups**: Set up cron jobs to automate periodic backups.
- **Retention Policy**: Define how many backups to keep and how long to retain them.
- **Restore Capability**: Restore specific volumes or all volumes from a backup archive on the remote server.

## How It Works

1. The program detects Docker volumes by matching folder names in `/backup` to the Docker volume names.
2. It stops containers using these volumes, archives the volume, restarts the containers, and uploads the backup to a
   designated server via SSH.
3. Any mounted directory (even non-Docker volumes) can also be backed up if mounted to `/backup`.
4. Backups can be scheduled using cron or executed manually.
5. Restores can be performed for all or specific volumes from any backup.
6. Before performing a restore, the program automatically creates a backup of the current state to prevent accidental
   data loss.

## Example `docker-compose.yml` Setup

```yaml
services:
  backup:
    image: kobesada/docker-volumes-backup-restore:latest
    env_file: backup.env
    volumes:
      - db:/backup/my_db
      - media:/backup/my_media
      - ~/.ssh/id_rsa:/app/.ssh/id_rsa
      - /var/run/docker.sock:/var/run/docker.sock

volumes:
  db:
    external: true
    name: my_db
  media:
    external: true
    name: my_media
```

### Explanation

- **Volume Handling**: If a folder in `/backup` matches the name of a Docker volume (e.g., `/backup/my_media`
  corresponds to the `my_media` volume name), the program will stop containers using that volume, archive the data, and
  restart the containers to preserve data consistency.
- **Non-Docker Volumes**: You can back up other folders by mounting them to `/backup`. If backing up a single file, wrap
  it in a folder before mounting.

## Environment Variables

### Server Configuration

Define these environment variables in your `backup.env` file:

```bash
SERVER_IP=123.123.123.123
SERVER_PORT=22
SERVER_USER=root
SERVER_DIRECTORY=/path/to/my/backup/folder
```

### Action Configuration

- **ACTION**: Set to either `backup` to create a backup or `restore` to restore a backup.

### Backup Configuration (for `backup` action)

- **BACKUP_CRON**: Optional. Defines the cron schedule for backups (e.g., `'0 2 * * * * *'`). If not set, the backup
  runs only once.
- **BACKUP_RETENTION_COUNT**: Optional. Defines the maximum number of backups to keep. The latest backup is always
  retained. If not set, backups are kept indefinitely.
- **BACKUP_RETENTION_PERIOD_IN_DAYS**: Optional. Defines how many days to retain backups. Older backups are
  automatically deleted. If not set, backups are not deleted based on age.

### Restore Configuration (for `restore` action)

- **BACKUP_TO_BE_RESTORED**: Specify `'latest'` to restore the most recent backup, or provide the name of a specific
  backup (e.g., `backup-2024-09-10T16-02-47.tar.gz`).
- **VOLUME_TO_BE_RESTORED**: Specify `'all'` to restore all volumes, or list specific volumes (e.g., `'my_db'`, or
  `'my_db, my_media'`).

## Running the Program

1. Clone the repository and build the Docker image:
   ```bash
   docker-compose build
   ```

2. Start the backup service:
   ```bash
   docker-compose up
   ```

3. Adjust your environment variables in the `backup.env` file to configure backup retention, server details, and whether
   you want to run a backup or restore.

## Example Scenarios

### Scheduled Backups

To run backups every day at 2 AM, use the following `backup.env`:

```bash
SERVER_IP=123.123.123.123
SERVER_PORT=22
SERVER_USER=root
SERVER_DIRECTORY=/path/to/my/backup/folder

ACTION=backup
BACKUP_CRON='0 2 * * *'
```

### Setup Backup Rotation

To run backups daily at 2 AM, with a maximum of 12 backups retained and backups older than 7 days deleted:

```bash
SERVER_IP=123.123.123.123
SERVER_PORT=22
SERVER_USER=root
SERVER_DIRECTORY=/path/to/my/backup/folder

ACTION=backup
BACKUP_CRON='0 2 * * *'
BACKUP_RETENTION_COUNT=12
BACKUP_RETENTION_PERIOD_IN_DAYS=7
```

### One-Time Backup

To run a single backup, remove the `BACKUP_CRON`:

```bash
SERVER_IP=123.123.123.123
SERVER_PORT=22
SERVER_USER=root
SERVER_DIRECTORY=/path/to/my/backup/folder

ACTION=backup
```

### Restoring a Backup

To restore all volumes from the latest backup:

```bash
SERVER_IP=123.123.123.123
SERVER_PORT=22
SERVER_USER=root
SERVER_DIRECTORY=/path/to/my/backup/folder

ACTION=restore
BACKUP_TO_BE_RESTORED=latest
VOLUME_TO_BE_RESTORED=all
```

To restore specific volumes from a specific backup:

```bash
SERVER_IP=123.123.123.123
SERVER_PORT=22
SERVER_USER=root
SERVER_DIRECTORY=/path/to/my/backup/folder

ACTION=restore
BACKUP_TO_BE_RESTORED=backup-2024-09-10T16-02-47.tar.gz
VOLUME_TO_BE_RESTORED='my_db, my_media'
```

## License

This project is licensed under the MIT License.

---
