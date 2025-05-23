# /// script
# requires-python = ">=3.9"
# dependencies = [
#     "httpx",
# ]
# ///
import time
from datetime import datetime, timezone

import httpx  # pyright: ignore

API_URL = "https://api.mangaupdates.com/v1/publishers/search"
TYPE = "English"
PER_PAGE = 40
PUBLISHERS: list[str] = []
PAYLOAD = {
    "page": 1,
    "perpage": PER_PAGE,
    "orderby": "type",
    "pending": False,
}
PAGES = range(4, 12)


with httpx.Client() as client:
    for page in PAGES:
        PAYLOAD["page"] = page
        response = client.post(API_URL, json=PAYLOAD).raise_for_status().json()
        for result in response["results"]:
            if result["record"]["type"] == "English":
                publisher = result["record"]["name"]
                if (
                    publisher
                    not in (
                        "J-Novel Club",  # Handled manually
                        "Kodansha International",  # Handled manually
                        "Seven Seas Entertainment",  # Handled manually
                        "Kodansha International",  # Handled manually
                        "Yen Press",  # Handled manually
                        "Square Enix USA",  # Handled manually
                    )
                    and len(publisher) > 5  # Avoid False positives from short names
                ):
                    PUBLISHERS.append(publisher)

        time.sleep(1)

with open("src/publishers.rs", "w", encoding="utf-8") as file:
    file.write("// This file is autogenerated by `scripts/generate-publishers.py`.\n")
    file.write("// Do not edit it by hand.\n")
    file.write(f"// Last updated at: {datetime.now(timezone.utc)}\n\n")
    file.write(f"pub(crate) const KNOWN_PUBLISHERS: &[&str; {len(PUBLISHERS)}] = &[\n")
    for publisher in PUBLISHERS:
        file.write(f'    "{publisher}",\n')
    file.write("];\n")
