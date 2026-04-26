"""Create satellites table and seed five curated rows.

Revision ID: 0001
Revises:
Create Date: 2026-04-26 00:00:00.000000

"""

from __future__ import annotations

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

# revision identifiers, used by Alembic.
revision: str = "0001"
down_revision: str | None = None
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "satellites",
        sa.Column("id", sa.Integer(), autoincrement=True, nullable=False),
        sa.Column("norad_id", sa.Integer(), nullable=False),
        sa.Column("name", sa.Text(), nullable=False),
        sa.Column("kind", sa.Text(), nullable=False),
        sa.Column("description", sa.Text(), nullable=True),
        sa.Column(
            "added_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.CheckConstraint(
            "kind IN ('station','telescope','comms','gps','weather','other')",
            name="satellites_kind_check",
        ),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("norad_id"),
    )

    # Seed the five curated satellites from spec.md § Curated satellite dropdown.
    op.execute(
        sa.text(
            """
            INSERT INTO satellites (norad_id, name, kind, description) VALUES
            (25544, 'ISS (ZARYA)',              'station',   'International Space Station'),
            (20580, 'Hubble Space Telescope',   'telescope', 'NASA/ESA space telescope in LEO'),
            (44713, 'Starlink-1007',            'comms',     'SpaceX Starlink LEO constellation satellite'),
            (36585, 'GPS BIIF-1 (NAVSTAR-65)',  'gps',       'GPS Block IIF MEO navigation satellite'),
            (33591, 'NOAA-19',                  'weather',   'NOAA polar-orbiting weather satellite')
            """
        )
    )


def downgrade() -> None:
    op.drop_table("satellites")
