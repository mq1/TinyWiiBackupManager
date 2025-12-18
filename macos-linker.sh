#!/bin/sh
# this is a wrapper to adapt ld64 to gnu style arguments

declare -a args=()
for arg in "$@"
do
    # options for linker
    if [[ $arg == "-Wl,"* ]]; then
        IFS=',' read -r -a options <<< "${arg#-Wl,}"
        for option in "${options[@]}"
        do
            if [[ $option == "-plugin="* ]] || [[ $option == "-plugin-opt=mcpu="* ]]; then
                # ignore -lto_library and -plugin-opt=mcpu
                :
            elif [[ $option == "-plugin-opt=O"* ]]; then
                # convert -plugin-opt=O* to --lto-CGO*
                args[${#args[@]}]="-Wl,--lto-CGO${option#-plugin-opt=O}"
            else
                # pass through other arguments
                args[${#args[@]}]="-Wl,$option"
            fi
        done

    else
        # pass through other arguments
        args[${#args[@]}]="$arg"
    fi
done

# use clang to call ld64
exec ${CC} -v "${args[@]}"

