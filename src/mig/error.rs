/// This module contains types for errors, which may happen during
/// parsing and matching of messages.
use crate::mig::either::Either;
use std::fmt;
use serde::{Deserialize, Serialize};
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct InterchangeError {
    pub pos: usize,
    pub service_segment_error: Option<ServiceSegmentError>,
    pub message_errors: Vec<MessageError>
}

#[derive(Debug, Clone)]
pub struct MessageError {
    pub pos: usize,
    pub service_segment_error: Option<ServiceSegmentError>,
    pub segment_errors: Vec<SegmentError>
}

#[derive(Debug, Clone)]
pub struct ServiceSegmentError {
    pub tag: String,
    pub error: Either<CompositeError, DataElementError>,
}

#[derive(Debug, Clone)]
pub struct SegmentError {
    pub pos: usize,
    pub syntax_error: Option<SyntaxError>,
    pub errors: Vec<Either<CompositeError, DataElementError>>,
}

#[derive(Debug, Clone)]
pub struct CompositeError {
    pub pos: usize,
    pub syntax_error: Option<SyntaxError>,
    pub errors: Vec<DataElementError>,
}

impl CompositeError {
    pub fn syntax_error(pos: usize, syntax_error: SyntaxError) -> Self {
        CompositeError {
            pos,
            syntax_error: Some(syntax_error),
            errors: vec![]
        }
    }
}

#[derive(Debug, Clone)]
pub struct DataElementError {
    pub pos: usize,
    pub syntax_error: SyntaxError,
}

impl DataElementError {
    pub fn new(pos: usize, syntax_error: SyntaxError) -> Self {
        DataElementError {
            pos,
            syntax_error
        }
    }
}

impl fmt::Display for DataElementError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.pos, self.syntax_error)
    }
}

/// A `SyntaxError` is one of the error codes defined in a CONTRL message.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct SyntaxError {
    code: u64,
    name: &'static str,
    message: &'static str,
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}\n\n{}", self.get_code(), self.get_name(), self.get_message())
    }
}

#[allow(dead_code)]
impl SyntaxError {
    pub fn syntax_version_or_level_not_supported() -> Self {
        SyntaxError {
            code: 2,
            name: "Syntax-Version oder -ebene nicht unterstützt",
            message: "Mitteilung, dass die Syntax-Version und/oder -ebene vomEmpfänger nicht unterstützt wird."
        }
    }

    pub fn receiver_is_not_actual_receiver() -> Self {
        SyntaxError {
            code: 7,
            name: "Empfänger der Übertragungsdatei ist nicht der tatsächliche Empfänger",
            message: "Mitteilung, dass der Empfänger der Übertragungsdatei(S003) \
                      vom tatsächlichen Empfänger abweicht."
        }
    }

    pub fn invalid_value() -> Self {
        SyntaxError {
            code: 12,
            name: "Ungültiger Wert",
            message: "Mitteilung, dass der Wert eines einfachen Datenelements, \
                      einer Datenelementgruppe oder eines Gruppendatenelements \
                      nicht den entsprechendenSpezifikationen entspricht."
        }
    }

    pub fn missing() -> Self {
        SyntaxError {
            code: 13,
            name: "Fehlt",
            message: "Mitteilung, dass ein mit M oder R gekennzeichnetes Service-oder \
                     Nutzdaten-Segment, Datenelement, eineDatenelementgruppe oderein \
                     Gruppendatenelement fehlt."
        }
    }

    pub fn not_supported_at_this_position() -> Self {
        SyntaxError {
            code: 15,
            name: "Nicht unterstützt an dieser Position",
            message: "Mitteilung, dass der Empfänger die Verwendung des Typs von \
                      Segment, an der identifizierten Position nicht unterstützt."
                    
        }
    }

    pub fn too_many_parts() -> Self {
        SyntaxError {
            code: 16,
            name: "Zuviele Bestandteile",
            message: "Mitteilung, dass das identifizierte Segment zu vieleDatenelemente \
                     oder Datenelementgruppen enthält."
                    
        }
    }

    pub fn invalid_service_chars() -> Self {
        SyntaxError {
            code: 20,
            name: "Zeichen ungültig als Service-Zeichen",
            message: "Mitteilung, dass ein im UNA angezeigtes Zeichen als \
                     Service-Zeichen ungültig ist."
                    
        }
    }

    pub fn invalid_characters() -> Self {
        SyntaxError {
            code: 21,
            name: "Ungültige(s) Zeichen",
            message: "Mitteilung, dass ein oder mehrere in der Übertragungsdateiverwendete \
                      Zeichen nach der definierten Syntax-Ebene imSegment UNB ungültig sind. \
                      Das ungültige Zeichen ist Teilder Bezugsebene oder folgt unmittelbar dem \
                      identifizierten Teil der Übertragungsdatei."
                    
        }
    }

    pub fn unknown_sender() -> Self {
        SyntaxError {
            code: 23,
            name: "Ungültige(s) Zeichen",
            message: "Mitteilung, dass ein oder mehrere in der Übertragungsdateiverwendete \
                      Zeichen nach der definierten Syntax-Ebene imSegment UNB ungültig sind. \
                      Das ungültige Zeichen ist Teilder Bezugsebene oder folgt unmittelbar dem \
                      identifizierten Teil der Übertragungsdatei."
                    
        }
    }

    pub fn test_not_supported() -> Self {
        SyntaxError {
            code: 25,
            name: "Test-Kennzeichen nicht unterstützt",
            message: "Mitteilung, dass die Test-Verarbeitung für die angegebene Übertragungsdatei, \
                     Nachrichtengruppe oder Nachricht nichtdurchgeführt werden konnte."
                    
        }
    }

    pub fn duplicate_found() -> Self {
        SyntaxError {
            code: 26,
            name: "Duplikat gefunden",
            message: "Mitteilung, dass ein mögliches Duplikat einer früherempfangenen \
                      Übertragungsdatei gefunden wurde. Diefrühere Übertragung kann \
                      zurückgewiesen worden sein (Datenaustauschreferenz des Absenders \
                      bei Empfängerbereits bekannt)."
                    
        }
    }

    pub fn references_not_equal() -> Self {
        SyntaxError {
            code: 28,
            name: "Referenzen stimmen nicht überein",
            message: "Mitteilung, dass die Prüfreferenzen im Segment UNB nicht denen in \
                      den Segment UNZ entsprechen."
                     
        }
    }

    pub fn counter_not_equal() -> Self {
        SyntaxError {
            code: 29,
            name: "Kontrollzähler entspricht nicht der Anzahlempfangender Fälle",
            message: "Mitteilung, dass die Anzahl der Nachrichten nichtder imSegment \
                     UNZ angegebenen Anzahl entspricht."
                    
        }
    }

    pub fn lower_levels_empty() -> Self {
        SyntaxError {
            code: 32,
            name: "Tiefere Ebene leer",
            message: "Mitteilung, dass die Übertragungsdatei keine Nachrichtenenthielt."
        }
    }

    pub fn too_many_segment_repetitions() -> Self {
        SyntaxError {
            code: 35,
            name: "Zu viele Segment-Wiederholungen",
            message: "Mitteilung, dass ein Segment zu oft wiederholt wurde"
                     
        }
    }

    pub fn too_many_segmentgroup_repetitions() -> Self {
        SyntaxError {
            code: 36,
            name: "Zu viele Segmentgruppen-Wiederholungen",
            message: "Mitteilung, dass eine Segmentgruppe zu oft wiederholt wurde."
        }
    }

    pub fn invalid_format() -> Self {
        SyntaxError {
            code: 37,
            name: "Ungültige Zeichenart",
            message: "Mitteilung, dass ein oder mehrere numerische Zeichen in einem \
                      alphabetischen (Gruppen-)Datenelement oder einoder mehrere \
                      alphabetische Zeichen in einem numerischen (Gruppen-)Datenelement \
                      verwendet wurden."
                     
        }
    }

    pub fn missing_digit_in_front_of_decimal() -> Self {
        SyntaxError {
            code: 38,
            name: "Fehlende Ziffer vor dem Dezimalzeichen",
            message: "Mitteilung, dass vor einem Dezimalzeichen nicht eine oder mehrere \
                     Ziffern stehen."
        }
    }

    pub fn data_element_too_long() -> Self {
        SyntaxError {
            code: 39,
            name: "Datenelement zu lang",
            message: "Mitteilung, dass die Länge eines empfangenen Datenelements die \
                     maximale Länge nach derDatenelementbeschreibung überschreitet."
        }
    }

    pub fn data_element_too_short() -> Self {
        SyntaxError {
            code: 40,
            name: "Datenelement zu lang",
            message: "Mitteilung, dass die Länge eines empfangenen Datenelements die \
                     maximale Länge nach der Datenelementbeschreibung überschreitet."
        }
    }


    pub fn get_code(&self) -> u64 {
        self.code
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn get_message(&self) -> &'static str {
        self.message
    }

}
