"""Create tles table and tles_latest_idx index.

Revision ID: 0002
Revises: 0001
Create Date: 2026-04-26 00:00:00.000001

"""

from __future__ import annotations

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op

# revision identifiers, used by Alembic.
revision: str = "0002"
down_revision: str | None = "0001"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "tles",
        sa.Column("id", sa.BigInteger(), autoincrement=True, nullable=False),
        sa.Column("norad_id", sa.Integer(), nullable=False),
        sa.Column("line1", sa.Text(), nullable=False),
        sa.Column("line2", sa.Text(), nullable=False),
        sa.Column("epoch", sa.DateTime(timezone=True), nullable=False),
        sa.Column(
            "fetched_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.Column("source", sa.Text(), server_default=sa.text("'celestrak'"), nullable=False),
        sa.ForeignKeyConstraint(["norad_id"], ["satellites.norad_id"]),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("norad_id", "epoch"),
    )
    op.create_index(
        "tles_latest_idx",
        "tles",
        ["norad_id", sa.text("epoch DESC")],
    )


def downgrade() -> None:
    op.drop_index("tles_latest_idx", table_name="tles")
    op.drop_table("tles")
