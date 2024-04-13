# discalen

A bot to notify your discord server about upcoming events

## Commands

Note: any command can return error due to Google API long responses. Don't panic if see a red message.

- `/create_calendar` - create a calendar (admins only).
- `/delete_calendar` - delete server calendars (admins only).
- `/list_events` - list all the events, show calendar url.
- `/create_event <label> <date>` - create an event (admins only).
- `/delete_event <label>` - delete an event (admins only).
- `/set_event_channel` - make the event channel receive event notifications (admins only).
- `/ping` - is bot alive?

## Testing in Discord

There's the link to add the bot: https://discord.com/oauth2/authorize?client_id=1225950004909314170&permissions=2048&scope=bot

Add the bot to a server and play with its functionality.

## Building locally (if you are experienced enough, and really want it running locally)

### Prerequisites

- A Google service account with Calendar API
- A Discord app with message content intent
- docker
- psql
- cargo sqlx

### Step-by-step

Note: not sure if the `init_db.sh` script will work on Windows, tested on Linux

1. Insert your Google service account key into `secrets/google-sa-secret.json`
2. Insert your Discord App token into `secrets/discord-token.txt`
3. Insert your postgres db password into `secrets/db_password.txt`
4. Configure the app config, mainly db connection (defaults should be ok)
5. Create `.env` file with `DATABASE_URL` field (`DATABASE_URL="postgres://..."`) pointing to your local db
6. `chmod +x scripts/inti_db.sh` (grant execution permissions)
7. `./scripts/init_db.sh`
8. `cargo r`
