import sqlite3


with sqlite3.connect('../data/mir_database.db') as connection:
        cursor = connection.cursor()
        #Create table
        create_table_query = '''
        CREATE TABLE IF NOT EXISTS MiRNAs (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        sequence TEXT NOT NUll
        );
        '''
        cursor.execute(create_table_query)
        connection.commit()
        
        #Inserting the miRNA name, sequence data
        insert_query = '''
        INSERT INTO MiRNAs (name, sequence) 
        VALUES (?, ?);
        '''
        miRNA_data = []
        #TODO remove assumptions
        with open("../data/mature.fa") as f:
            l1 = f.readline()
            l2 = f.readline()
            while l1 and l2:
                miRNA_data += [(l1.strip(), l2.strip())]
                l1 = f.readline()
                l2 = f.readline()
                

        cursor.executemany(insert_query, miRNA_data)
        connection.commit()
