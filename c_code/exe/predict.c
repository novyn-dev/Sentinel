#include <LIEF/PE/Binary.h>
#include <LIEF/PE/Import.h>
#include <math.h>
#include <stddef.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <LIEF/LIEF.h>
#include <stdbool.h>
#include <xgboost/c_api.h>
#include "../helper.h"

bool lief_pe_has_signatures(Pe_Binary_t *pe_binary);
bool lief_pe_has_imports(Pe_Binary_t *pe_binary);
bool lief_pe_has_sections(Pe_Binary_t *pe_binary);

typedef unsigned int uint;

typedef struct {
    int start;
    int end;
} Range;

static float* histogram(const unsigned char* data, size_t len, uint bins, Range range) {
    float* freq = calloc(bins, sizeof(float));
    if (!freq) return NULL;

    for (size_t i = 0; i < len; i++) {
        if (data[i] >= range.start && data[i] < range.end)
            freq[data[i] - range.start]++;
    }

    return freq;
}

static float calculate_entropy(const unsigned char* data, size_t len) {
    Range range = { 0, 256 };
    float* counts = histogram(data, len, 256, range);

    float entropy = 0.0;
    for (int i = 0; i < 256; i++) {
        if (counts[i] > 0) {
            float prob = (float)counts[i] / len;
            entropy -= prob * log2f(prob);
        }
    }

    free(counts);
    return entropy;
}

static float* calculate_byte_entropy(const unsigned char* data, uint data_len, uint block_size, int* out_size_entropies) {
    float* entropies = malloc(sizeof(float) * (data_len / block_size + 1));
    int size_entropies = 0;

    if (entropies == NULL) {
        printf("Memory allocation `entropies` failed");
    }

    for (int i = 0; i < data_len; i += block_size) {
        int block_len = (i + block_size < data_len) ? block_size : (data_len - i);
        float entropy = calculate_entropy(data, block_len);
        entropies[size_entropies] = entropy;
        size_entropies++;
    }
    *out_size_entropies = size_entropies;
    return entropies;
}

static char** extract_strings(const unsigned char* data, int data_len, int* out_count) {
    char** results = NULL;
    int count = 0;
    size_t i = 0;

    int min_len = 4;

    while (i < data_len) {
        size_t start = i;

        // scan while chars are printable ascii
        while(data[i] >= 0x20 && data[i] <= 0x7e) {
            i++;
        }

        size_t len = i - start;
        if (len >= min_len) {
            results = realloc(results, sizeof(char*) * (count + 1));
            char *s = malloc(len + 1);
            memcpy(s, &data[start], len);
            s[len] = '\0';
            results[count++] = s;
        }

        // skip non-printable ascii chars
        i++;
    }

    *out_count = count;
    return results;
}

static float* extract_features_from_file_pe(char* filepath) {
    float* result = malloc(sizeof(float) * 9); // 9 features
    size_t result_size = 0;
    int n_strings;
    Pe_Binary_t *binary = pe_parse(filepath);

    if (binary == NULL) {
        perror("bruh");
    }

    // open file
    FILE *f = fopen(filepath, "rb");
    if (f == NULL) {
        perror("Failed to open the file");
    }

    // get file size
    fseek(f, 0, SEEK_END);
    int filesize = ftell(f);
    rewind(f);

    unsigned char* data = malloc(filesize);
    if (data == NULL) {
        perror("Failed to allocate memory");
        fclose(f);
    }

    size_t bytes_read = fread(data, 1, filesize, f);
    fclose(f);

    char** strings = extract_strings(data, bytes_read, &n_strings);
    Range range = { 0, 256 };
    float* hist = histogram(data, bytes_read, 256, range);

    float entropy = calculate_entropy(data, bytes_read);

    int byte_entropy_len;
    float* byte_entropy = calculate_byte_entropy(data, bytes_read, 1024, &byte_entropy_len);

    int avlength = 0;
    if (strings) {
        if (n_strings > 0) {
            size_t total_strlen = 0;
            for (size_t i = 0; i < n_strings; i++) {
                total_strlen += strlen(strings[i]);
            }
            avlength = total_strlen / n_strings;
        }
    }

    int hist_len = 256; // allocated 256 bins in histogram function

    result[result_size++] = (float)bytes_read;
    result[result_size++] = lief_pe_has_imports(binary);
    result[result_size++] = lief_pe_has_signatures(binary);
    result[result_size++] = lief_pe_has_sections(binary);
    result[result_size++] = mean(hist, hist_len);
    result[result_size++] = std(hist, hist_len);
    result[result_size++] = max(hist, hist_len);
    result[result_size++] = entropy;
    result[result_size++] = n_strings;

    for (int i = 0; i < n_strings; i++) {
        free(strings[i]);
    }
    free(strings);
    free(hist);
    free(byte_entropy);
    return result;
}

void predict_malware_pe(char* filepath, char* model_path) {
    BoosterHandle booster;
    XGBoosterCreate(NULL, 0, &booster);
    XGBoosterLoadModel(booster, model_path);

    float* features = extract_features_from_file_pe(filepath);
    DMatrixHandle features_mat;
    XGDMatrixCreateFromMat(features, 1, 9, 0, &features_mat);

    // prediction
    // the config and 
    char const config[] =
        "{\"training\": false, \"type\": 0, "
        "\"iteration_begin\": 0, \"iteration_end\": 0, \"strict_shape\": false}";
    /* Shape of output prediction */
    uint64_t const* out_shape;
    /* Dimension of output prediction */
    uint64_t out_dim;
    /* Pointer to a thread local contiguous array, assigned in prediction function. */
    float const* out_result = NULL;
    XGBoosterPredictFromDMatrix(booster, features_mat, config, &out_shape, &out_dim, &out_result);

    float pred = out_result[0];
    bool is_malware = pred > 0.099;

    if (is_malware == true) {
        printf("It's a malware\n");
    } else {
        printf("It's not a malware\n");
    }

    XGBoosterFree(booster);
    XGDMatrixFree(features_mat);
    free(features);
}

// this is only for testing purposes lol ;-;
