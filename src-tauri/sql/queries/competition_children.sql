SELECT Competition.* FROM Competition
INNER JOIN Season ON Season.comp_id = Competition.id
INNER JOIN CompRelation
ON Competition.id = CompRelation.child_id
WHERE CompRelation.parent_id = 7
GROUP BY Season.comp_id
ORDER BY Season.id DESC, Season.start_date ASC