//
// fclose.cpp
//
//      Copyright (c) Microsoft Corporation.  All rights reserved.
//
// Defines fclose(), which closes a stdio stream.
//
#include <corecrt_internal_stdio.h>



// Closes a stdio stream after flushing the stream and freeing any buffer
// associated with the stream (unless the buffer was set with setbuf).  Returns
// zero on success; EOF on failure (e.g., if the flush fails, or it is not a
// valid fileo or the file is not open, etc.).
extern "C" int __cdecl fclose(FILE* const public_stream)
{
    __crt_stdio_stream const stream(public_stream);

    _VALIDATE_RETURN(stream.valid(), EINVAL, EOF);

    // If the stream is backed by a string, it requires no synchronization,
    // flushing, etc., so we can simply free it, which resets all of its
    // data to the defaults.
    if (stream.is_string_backed())
    {
        __acrt_stdio_free_stream(stream);
        return EOF;
    }

    int return_value = 0;

    _lock_file(stream.public_stream());
    __try
    {
        return_value = _fclose_nolock(stream.public_stream());
    }
    __finally
    {
        _unlock_file(stream.public_stream());
    }

    return return_value;
}



extern "C" int __cdecl _fclose_nolock(FILE* const public_stream)
{
    __crt_stdio_stream const stream(public_stream);

    _VALIDATE_RETURN(stream.valid(), EINVAL, EOF);

    int result = EOF;

    if (stream.is_in_use())
    {
        result = __acrt_stdio_flush_nolock(stream.public_stream());
        __acrt_stdio_free_buffer_nolock(stream.public_stream());

        if (_close(_fileno(stream.public_stream())) < 0)
        {
            result = EOF;
        }
        else if (stream->_tmpfname != nullptr)
        {
            _free_crt(stream->_tmpfname);
            stream->_tmpfname = nullptr;
        }
    }

    __acrt_stdio_free_stream(stream);

    return result;
}
