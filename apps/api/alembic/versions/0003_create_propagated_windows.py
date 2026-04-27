"""Create propagated_windows table and propagated_windows_lookup_idx.

Revision ID: 0003
Revises: 0002
Create Date: 2026-04-26 00:00:00.000002

"""

from __future__ import annotations

from collections.abc import Sequence

import sqlalchemy as sa
from alembic import op
from sqlalchemy.dialects.postgresql import JSONB

# revision identifiers, used by Alembic.
revision: str = "0003"
down_revision: str | None = "0002"
branch_labels: str | Sequence[str] | None = None
depends_on: str | Sequence[str] | None = None


def upgrade() -> None:
    op.create_table(
        "propagated_windows",
        sa.Column("id", sa.BigInteger(), autoincrement=True, nullable=False),
        sa.Column("hash", sa.Text(), nullable=False),
        sa.Column("tle_id", sa.BigInteger(), nullable=False),
        sa.Column("start_at", sa.DateTime(timezone=True), nullable=False),
        sa.Column("duration_s", sa.Integer(), nullable=False),
        sa.Column("step_s", sa.Integer(), nullable=False),
        sa.Column("frame", sa.Text(), nullable=False),
        sa.Column("include_velocity", sa.Boolean(), nullable=False, server_default=sa.text("true")),
        sa.Column("samples", JSONB(), nullable=False),
        sa.Column(
            "computed_at",
            sa.DateTime(timezone=True),
            server_default=sa.text("now()"),
            nullable=False,
        ),
        sa.ForeignKeyConstraint(["tle_id"], ["tles.id"], ondelete="CASCADE"),
        sa.PrimaryKeyConstraint("id"),
        sa.UniqueConstraint("hash"),
    )
    op.create_index(
        "propagated_windows_lookup_idx",
        "propagated_windows",
        ["tle_id", "start_at"],
    )


def downgrade() -> None:
    op.drop_index("propagated_windows_lookup_idx", table_name="propagated_windows")
    op.drop_table("propagated_windows")
