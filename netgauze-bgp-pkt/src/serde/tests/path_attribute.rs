// Copyright (C) 2022-present The NetGauze Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//    http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or
// implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::{
    path_attribute::{Origin, PathAttributeLength, UndefinedOrigin},
    serde::{
        deserializer::path_attribute::{LocatedOriginParsingError, OriginParsingError},
        serializer::path_attribute::OriginWritingError,
    },
};
use netgauze_parse_utils::{
    test_helpers::{
        test_parse_error_with_one_input, test_parsed_completely_with_one_input, test_write,
        test_write_with_one_input,
    },
    Span,
};

#[test]
fn test_origin_value() -> Result<(), OriginWritingError> {
    let good_igp_wire = [0x01, 0x00];
    let good_egp_wire = [0x01, 0x01];
    let good_incomplete_wire = [0x01, 0x02];
    let bad_zero_length_wire = [0x0, 0x02];
    let bad_long_length_wire = [0x2, 0x02];
    let bad_invalid_code_wire = [0x1, 0x03];

    let igp = Origin::IGP;
    let egp = Origin::EGP;
    let incomplete = Origin::Incomplete;
    let bad_zero_length = LocatedOriginParsingError::new(
        Span::new(&bad_zero_length_wire),
        OriginParsingError::InvalidOriginLength(PathAttributeLength::U8(0)),
    );

    let bad_long_length = LocatedOriginParsingError::new(
        Span::new(&bad_long_length_wire),
        OriginParsingError::InvalidOriginLength(PathAttributeLength::U8(2)),
    );

    let bad_invalid_code = LocatedOriginParsingError::new(
        unsafe { Span::new_from_raw_offset(1, &bad_invalid_code_wire[1..]) },
        OriginParsingError::UndefinedOrigin(UndefinedOrigin(3)),
    );

    test_parsed_completely_with_one_input(&good_igp_wire, false, &igp);
    test_parsed_completely_with_one_input(&good_egp_wire, false, &egp);
    test_parsed_completely_with_one_input(&good_incomplete_wire, false, &incomplete);
    test_parse_error_with_one_input::<'_, Origin, bool, LocatedOriginParsingError<'_>>(
        &bad_zero_length_wire,
        false,
        &bad_zero_length,
    );
    test_parse_error_with_one_input::<'_, Origin, bool, LocatedOriginParsingError<'_>>(
        &bad_long_length_wire,
        false,
        &bad_long_length,
    );
    test_parse_error_with_one_input::<'_, Origin, bool, LocatedOriginParsingError<'_>>(
        &bad_invalid_code_wire,
        false,
        &bad_invalid_code,
    );

    test_write_with_one_input(&igp, false, &good_igp_wire)?;
    test_write_with_one_input(&egp, false, &good_egp_wire)?;
    test_write_with_one_input(&incomplete, false, &good_incomplete_wire)?;
    Ok(())
}
