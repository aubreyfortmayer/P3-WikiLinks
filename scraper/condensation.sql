UPDATE articles SET condensed_links = (SELECT id FROM articles art WHERE art.title = any (links));