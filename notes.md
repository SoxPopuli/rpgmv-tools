Saves are lzstring base64 encoded json blobs
Save metadata is stored in global.rpgsave, as an array, where index == save file count, assume index 0 is always null

- In `global.rpgsave` faces and characters array point to image files in `www/img/faces` and `www/img/characters` respectively
    - The subsequent integer refers to an index on the sprite sheet
    - Face sprites are 144x144
    - Character sprites are 48x48
