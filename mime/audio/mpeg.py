from tinytag import TinyTag

def archive(doc, file):
    tag = TinyTag.get(file)
    doc["album"]       = tag.album
    doc["albumartist"] = tag.albumartist   # album artist as string
    doc["artist"]      = tag.artist        # artist name as string
    doc["bitrate"]     = tag.bitrate       # bitrate in kBits/s
    doc["comment"]     = tag.comment       # file comment as string
    doc["composer"]    = tag.composer      # composer as string
    doc["disc"]        = tag.disc          # disc number
    doc["disc_total"]  = tag.disc_total    # the total number of discs
    doc["duration"]    = tag.duration      # duration of the song in seconds
    doc["filesize"]    = tag.filesize      # file size in bytes
    doc["genre"]       = tag.genre         # genre as string
    doc["samplerate"]  = tag.samplerate    # samples per second
    doc["title"]       = tag.title         # title of the song
    doc["track"]       = tag.track         # track number as string
    doc["track_total"] = tag.track_total   # total number of tracks as string
    doc["year"]        = int(tag.year)     # year or data as string
    doc["storage_format"] = 0
