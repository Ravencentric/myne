"""Parser for manga and light novel filenames."""

from typing import final

__all__ = ("Book", "__version__")

__version__: str = ...

@final
class Book:
    """Represents a book with various metadata attributes."""

    title: str
    """The title of the book."""
    digital: bool
    """Indicates if the book is in digital format or not."""
    edited: bool
    """Indicates if this version of the book has been edited."""
    compilation: bool
    """Indicates if this book is a compilation."""
    pre: bool
    """
    Indicates if the book is a pre-publication version or an official version
    with known issues or considered preliminary for other reasons.
    """
    revision: int
    """The revision number of the book."""
    volume: str | None
    """
    The volume identifier, if applicable (e.g., `1`, `1.5`, `1-2`, `43.5-45`).
    """
    chapter: str | None
    """
    The chapter identifier, if applicable (e.g., `1`, `15`, `100.5`, `10-12`, `55.5-60`).
    """
    group: str | None
    """The group identifier."""
    year: int | None
    """The publication year of the specific volume or chapter."""
    edition: str | None
    """The specific edition of the book."""
    extension: str | None
    """The file extension."""
    publisher: str | None
    """The publisher of the book."""

    def __new__(
        cls,
        *,
        title: str,
        digital: bool = False,
        edited: bool = False,
        compilation: bool = False,
        pre: bool = False,
        revision: int = 1,
        volume: str | None = None,
        chapter: str | None = None,
        group: str | None = None,
        year: int | None = None,
        edition: str | None = None,
        extension: str | None = None,
        publisher: str | None = None,
    ) -> Book:
        """
        Create a new Book instance.

        Returns
        -------
        Self
            A new instance of the Book class.

        """
    def __eq__(self, other: object) -> bool: ...
    def __hash__(self) -> int: ...
    @staticmethod
    def parse(filename: str, /) -> Book:
        """
        Parse a light novel or manga filename into a `Book` object.

        Parameters
        ----------
        filename : str
            The light novel or manga filename to parse.

        Returns
        -------
        Book
            A Book object populated with data extracted from the filename.

        """
    @staticmethod
    def from_json(data: str, /) -> Book:
        """
        Deserialize a Book instance from a JSON string.

        Parameters
        ----------
        data : str
            A JSON string representing a Book object.

        Returns
        -------
        Self
            A new Book object deserialized from the JSON string.

        Raises
        ------
        ValueError
            If the input string is not valid JSON or does not match the
            expected structure for a Book object.

        """

    def to_json(self) -> str:
        """
        Serialize the Book instance to a JSON string.

        Returns
        -------
        str
            A JSON string representation of the Book object.

        """
