## Przeglądarka ostrzega „Połączenie nie jest prywatne” — czy to normalne?

Tak. Przekierowane porty, Web UI i połączenia klientów współdzielą ten sam samodzielnie podpisany certyfikat serwera; samo szyfrowanie nie jest złamane. Zaimportuj certyfikat serwera do magazynu zaufanych certyfikatów urządzenia, aby na stałe wyciszyć ostrzeżenie.

## Czy Vibe Terminal jest darmowy?

Tak — klient/serwer na komputery stacjonarne i serwer headless są darmowe. Pobieraj i instaluj na tylu maszynach, ilu chcesz.

## Dokąd trafiają moje dane?

Nikąd, gdzie sam ich nie umieścisz. Sesje, dane uwierzytelniające (enc-v2:) i rozpoznawanie mowy żyją wyłącznie na twoich maszynach; klucze parowania nigdy nie podróżują jawnym tekstem.

## Czy można bezpiecznie udostępnić przekierowaną usługę?

Tak. Wbudowane udostępnienia wymagają tokenu: linki generowane przez klienta — udostępnienie `Host with HTTP server` i strona Settings — zawierają token dostępu, a połączenie bez ważnego tokenu i bez ważnej sesji jest zrywane, zanim cokolwiek zostanie wysłane. Przy pierwszej wizycie token jest wymieniany na bezpieczne ciasteczko sesyjne działające tylko dla tej usługi i znika z adresu URL — ticket Settings działa dokładnie raz i wygasa po minucie, a zatrzymanie lub ponowne uruchomienie udostępnienia unieważnia wcześniejsze linki i sesje. Traktuj pełny link jak dane uwierzytelniające i przekazuj go tylko osobom, którym ufasz. Przekierowanie do własnej usługi dodaje TLS z certyfikatem serwera, ale nie uwierzytelnianie — trzymaj uwierzytelnianie przed wrażliwymi usługami i zrywaj przekierowanie natychmiast z panelu przekierowywania portów, gdy skończysz.
