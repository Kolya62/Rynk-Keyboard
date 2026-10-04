package org.rynk.keyboard

import android.view.inputmethod.EditorInfo
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class EditorSyncTest {

    @Test
    fun trailingWordStopsAtNonLetters() {
        assertEquals("world", EditorSync.trailingWord("hello world"))
        assertEquals("", EditorSync.trailingWord("hello "))
        assertEquals("ru", EditorSync.trailingWord("mail.ru"))
        assertEquals("", EditorSync.trailingWord(""))
    }

    @Test
    fun composingWordMatchingEditorIsInSync() {
        assertTrue(EditorSync.isEngineInSync("say hello", "hello", ""))
        assertTrue(EditorSync.isEngineInSync("say Hello", "hello", ""))
    }

    @Test
    fun editorLaggingOrLeadingByInFlightEditsIsInSync() {
        // Engine typed "hel", editor has only applied "he" so far
        assertTrue(EditorSync.isEngineInSync("say he", "hel", ""))
        // Backspace in flight: engine already shortened the word
        assertTrue(EditorSync.isEngineInSync("say hello", "hell", ""))
        // First character still in flight
        assertTrue(EditorSync.isEngineInSync("say ", "h", ""))
    }

    @Test
    fun cursorMovedIntoAnotherWordIsOutOfSync() {
        // User typed "hello" and tapped into the middle of "wor|ld"
        assertFalse(EditorSync.isEngineInSync("big wor", "hello", ""))
        // User tapped after a space somewhere else
        assertFalse(EditorSync.isEngineInSync("big world ", "hello", ""))
    }

    @Test
    fun emptyComposingTracksLastCommittedWord() {
        assertTrue(EditorSync.isEngineInSync("hello ", "", "hello"))
        // Space commit still in flight
        assertTrue(EditorSync.isEngineInSync("hello", "", "hello"))
        // Cursor placed right after an unrelated word
        assertFalse(EditorSync.isEngineInSync("other", "", "hello"))
    }

    @Test
    fun enterActionFollowsImeOptions() {
        assertEquals(EditorInfo.IME_ACTION_SEARCH, EditorSync.enterAction(EditorInfo.IME_ACTION_SEARCH))
        assertEquals(EditorInfo.IME_ACTION_SEND, EditorSync.enterAction(EditorInfo.IME_ACTION_SEND))
        assertEquals(0, EditorSync.enterAction(EditorInfo.IME_ACTION_UNSPECIFIED))
        assertEquals(0, EditorSync.enterAction(EditorInfo.IME_ACTION_NONE))
        assertEquals(
            0,
            EditorSync.enterAction(EditorInfo.IME_ACTION_SEND or EditorInfo.IME_FLAG_NO_ENTER_ACTION)
        )
    }

    @Test
    fun wordsBackCoverTrailingSpacesAndPunctuation() {
        assertEquals(6, EditorSync.wordsBackLength("say hello ", 1))
        assertEquals(10, EditorSync.wordsBackLength("say hello ", 2))
        assertEquals(6, EditorSync.wordsBackLength("Hi, there!", 1))
        assertEquals(3, EditorSync.wordsBackLength("abc", 5))
        assertEquals(0, EditorSync.wordsBackLength("", 1))
    }
}
