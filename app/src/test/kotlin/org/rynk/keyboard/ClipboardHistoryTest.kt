package org.rynk.keyboard

import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test
import java.io.File
import java.nio.file.Files

class ClipboardHistoryTest {

    private fun history(): Pair<ClipboardHistory, File> {
        val dir = Files.createTempDirectory("clip").toFile()
        val file = File(dir, ClipboardHistory.FILE_NAME)
        return ClipboardHistory(file) to file
    }

    @Test
    fun newestFirstPinnedOnTopAndPersisted() {
        val (h, file) = history()
        h.add("first", 1000)
        h.add("second", 2000)
        h.add("multi\nline\twith tab", 3000)
        assertEquals(listOf("multi\nline\twith tab", "second", "first"), h.items(3000).map { it.text })

        h.togglePin(2, 3000) // "first"
        assertEquals("first", h.items(3000)[0].text)

        val reloaded = ClipboardHistory(file)
        assertEquals(h.items(3000), reloaded.items(3000))
    }

    @Test
    fun unpinnedEntriesExpire() {
        val (h, _) = history()
        h.ttlMillis = 1000
        h.add("old", 0)
        h.add("keep", 0)
        h.togglePin(0, 0)
        val pinned = h.items(0).first { it.pinned }.text
        val later = h.items(5000)
        assertEquals(listOf(pinned), later.map { it.text })
    }

    @Test
    fun copyingAgainMovesToTopAndKeepsPin() {
        val (h, _) = history()
        h.add("a", 1)
        h.add("b", 2)
        h.togglePin(1, 2) // "a"
        h.add("a", 3)
        val items = h.items(3)
        assertEquals(2, items.size)
        assertTrue(items.first { it.text == "a" }.pinned)
    }

    @Test
    fun capacityDropsOldestUnpinned() {
        val (h, _) = history()
        for (i in 0 until ClipboardHistory.MAX_ENTRIES + 5) h.add("item $i", i.toLong())
        val items = h.items(1000)
        assertEquals(ClipboardHistory.MAX_ENTRIES, items.size)
        assertEquals("item ${ClipboardHistory.MAX_ENTRIES + 4}", items.first().text)
    }

    @Test
    fun deleteAndClear() {
        val (h, _) = history()
        h.add("x", 1)
        h.add("y", 2)
        h.togglePin(0, 2) // "y"
        h.delete(1, 2) // "x"
        assertEquals(listOf("y"), h.items(2).map { it.text })
        h.clearUnpinned()
        assertEquals(1, h.items(2).size)
        h.clearAll()
        assertTrue(h.items(2).isEmpty())
    }

    @Test
    fun escapingRoundTrips() {
        val s = "a\\b\nc\td\re"
        assertEquals(s, ClipboardHistory.unescape(ClipboardHistory.escape(s)))
    }
}
