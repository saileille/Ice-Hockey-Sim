-- SQLite
SELECT * FROM TeamSeason
WHERE season_id = (
    SELECT id FROM Season
    WHERE comp_id = $1
    ORDER BY id DESC
    LIMIT 1
)