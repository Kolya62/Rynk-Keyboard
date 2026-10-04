package org.rynk.keyboard

import java.io.File

/**
 * Recently copied texts, kept on the device only (excluded from backups). Pinned entries stay
 * until unpinned; the others expire after [ttlMillis]. Newest first, pinned ones on top.
 */
class ClipboardHistory(private val file: File) {

    data class Entry(val text: String, val time: Long, val pinned: Boolean)

    private val entries = mutableListOf<Entry>()

    var ttlMillis: Long = DEFAULT_TTL_MILLIS

    init {
        load()
    }

    /** The list as shown in the panel; indices from the panel refer to it. */
    fun items(now: Long): List<Entry> {
        if (entries.removeAll { !it.pinned && now - it.time > ttlMillis }) save()
        return entries.sortedWith(compareByDescending<Entry> { it.pinned }.thenByDescending { it.time })
    }

    fun add(text: String, now: Long) {
        if (text.isBlank() || text.length > MAX_TEXT_LENGTH) return
        val existing = entries.firstOrNull { it.text == text }
        entries.removeAll { it.text == text }
        entries.add(Entry(text, now, existing?.pinned ?: false))
        // Oldest unpinned entries go first when full
        while (entries.size > MAX_ENTRIES) {
            val oldest = entries.filter { !it.pinned }.minByOrNull { it.time } ?: break
            entries.remove(oldest)
        }
        save()
    }

    fun togglePin(index: Int, now: Long) = withItem(index, now) { e ->
        entries[entries.indexOf(e)] = e.copy(pinned = !e.pinned)
    }

    fun delete(index: Int, now: Long) = withItem(index, now) { e -> entries.remove(e) }

    fun clearUnpinned() {
        entries.removeAll { !it.pinned }
        save()
    }

    fun clearAll() {
        entries.clear()
        save()
    }

    private fun withItem(index: Int, now: Long, action: (Entry) -> Unit) {
        val item = items(now).getOrNull(index) ?: return
        action(item)
        save()
    }

    private fun load() {
        entries.clear()
        val lines = runCatching { file.readLines() }.getOrNull() ?: return
        for (line in lines) {
            val parts = line.split('\t', limit = 3)
            if (parts.size != 3) continue
            val time = parts[0].toLongOrNull() ?: continue
            entries.add(Entry(unescape(parts[2]), time, parts[1] == "1"))
        }
    }

    private fun save() {
        runCatching {
            val tmp = File(file.parentFile, file.name + ".tmp")
            tmp.writeText(entries.joinToString("") { "${it.time}\t${if (it.pinned) 1 else 0}\t${escape(it.text)}\n" })
            if (!tmp.renameTo(file)) {
                file.delete()
                tmp.renameTo(file)
            }
        }
    }

    companion object {
        const val FILE_NAME = "clipboard_history.txt"
        const val MAX_ENTRIES = 50
        const val MAX_TEXT_LENGTH = 5000
        const val DEFAULT_TTL_MILLIS = 60 * 60 * 1000L

        internal fun escape(s: String) = s.replace("\\", "\\\\").replace("\n", "\\n").replace("\t", "\\t").replace("\r", "\\r")

        internal fun unescape(s: String): String {
            val out = StringBuilder(s.length)
            var i = 0
            while (i < s.length) {
                val c = s[i]
                if (c == '\\' && i + 1 < s.length) {
                    out.append(
                        when (s[i + 1]) {
                            'n' -> '\n'
                            't' -> '\t'
                            'r' -> '\r'
                            else -> s[i + 1]
                        }
                    )
                    i += 2
                } else {
                    out.append(c)
                    i++
                }
            }
            return out.toString()
        }
    }
}
